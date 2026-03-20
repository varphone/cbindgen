/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write;

use syn::ext::IdentExt;
use syn::UnOp;

use crate::bindgen::config::{Config, Language};
use crate::bindgen::declarationtyperesolver::DeclarationTypeResolver;
use crate::bindgen::dependencies::Dependencies;
use crate::bindgen::ir::{
    AnnotationSet, Cfg, ConditionWrite, Documentation, GenericParams, Item, ItemContainer, Path,
    PrimitiveType, Struct, ToCondition, Type,
};
use crate::bindgen::language_backend::LanguageBackend;
use crate::bindgen::library::Library;
use crate::bindgen::rename::{IdentifierType, RenameRule};
use crate::bindgen::writer::SourceWriter;
use crate::bindgen::Bindings;

fn member_to_ident(member: &syn::Member) -> String {
    match member {
        syn::Member::Named(ref name) => name.unraw().to_string(),
        syn::Member::Unnamed(ref index) => format!("_{}", index.index),
    }
}

fn escape_c_string_literal(bytes: &[u8]) -> Result<String, String> {
    if bytes.contains(&0) {
        return Err("string constants with interior NUL bytes are not supported".to_owned());
    }

    let mut escaped = String::with_capacity(bytes.len() + 2);
    escaped.push('"');
    for &byte in bytes {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            b'"' => escaped.push_str("\\\""),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            0x20..=0x7e => escaped.push(byte as char),
            _ => {
                write!(&mut escaped, "\\{:03o}", byte).unwrap();
            }
        }
    }
    escaped.push('"');
    Ok(escaped)
}

fn string_constant_type(ty: &syn::Type) -> Option<Type> {
    let syn::Type::Reference(reference) = ty else {
        return None;
    };

    if reference.mutability.is_some() {
        return None;
    }

    let syn::Type::Path(path) = reference.elem.as_ref() else {
        return None;
    };

    let ident = path.path.segments.last()?.ident.unraw();
    if ident != "str" && ident != "CStr" {
        return None;
    }

    Some(Type::Ptr {
        ty: Box::new(Type::Primitive(PrimitiveType::Char)),
        is_const: true,
        is_nullable: false,
        is_ref: false,
    })
}

// TODO: Maybe add support to more std associated constants.
pub(crate) fn to_known_assoc_constant(associated_to: &Path, name: &str) -> Option<String> {
    use crate::bindgen::ir::{IntKind, PrimitiveType};

    if name != "MAX" && name != "MIN" {
        return None;
    }

    let prim = PrimitiveType::maybe(associated_to.name())?;
    let prefix = match prim {
        PrimitiveType::Integer {
            kind,
            signed,
            zeroable: _,
        } => match kind {
            IntKind::B8 => {
                if signed {
                    "INT8"
                } else {
                    "UINT8"
                }
            }
            IntKind::B16 => {
                if signed {
                    "INT16"
                } else {
                    "UINT16"
                }
            }
            IntKind::B32 => {
                if signed {
                    "INT32"
                } else {
                    "UINT32"
                }
            }
            IntKind::B64 => {
                if signed {
                    "INT64"
                } else {
                    "UINT64"
                }
            }
            _ => return None,
        },
        _ => return None,
    };
    Some(format!("{prefix}_{name}"))
}

#[derive(Debug, Clone)]
pub struct LiteralStructField {
    pub value: Literal,
    pub cfg: Option<Cfg>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Expr(String),
    Path {
        associated_to: Option<(Path, String)>,
        name: String,
    },
    PostfixUnaryOp {
        op: &'static str,
        value: Box<Literal>,
    },
    BinOp {
        left: Box<Literal>,
        op: &'static str,
        right: Box<Literal>,
    },
    FieldAccess {
        base: Box<Literal>,
        field: String,
    },
    Struct {
        path: Path,
        export_name: String,
        fields: HashMap<String, LiteralStructField>,
    },
    Cast {
        ty: Type,
        value: Box<Literal>,
    },
}

impl Literal {
    fn resolve_relative_module_path(path: &str, current_module_path: &str) -> Option<String> {
        let segments = path.split("::").collect::<Vec<_>>();
        let first = *segments.first()?;

        if first != "self" && first != "super" {
            return None;
        }

        let mut module_segments = if current_module_path.is_empty() {
            Vec::new()
        } else {
            current_module_path.split("::").collect::<Vec<_>>()
        };

        let mut idx = 0;
        while idx < segments.len() {
            match segments[idx] {
                "self" => idx += 1,
                "super" => {
                    module_segments.pop();
                    idx += 1;
                }
                _ => break,
            }
        }

        module_segments.extend_from_slice(&segments[idx..]);
        Some(module_segments.join("::"))
    }

    fn hidden_constant_lookup_keys(
        associated_to: Option<&Path>,
        name: &str,
        struct_paths: &std::collections::HashSet<Path>,
    ) -> Vec<String> {
        let Some(base_key) = (match associated_to {
            Some(path) if !struct_paths.contains(path) => Some(format!("{}::{name}", path.name())),
            None => Some(name.to_owned()),
            _ => None,
        }) else {
            return Vec::new();
        };

        let mut keys = vec![base_key.clone()];
        if let Some(stripped) = base_key.strip_prefix("crate::") {
            keys.push(stripped.to_owned());
        }
        keys
    }

    fn resolve_path_head_aliases(&mut self, aliases: &HashMap<String, String>) {
        fn resolve_alias(path: &str, aliases: &HashMap<String, String>) -> Option<String> {
            let mut segments = path.split("::");
            let head = segments.next()?;
            let target = aliases.get(head)?;
            let tail = segments.collect::<Vec<_>>();
            if tail.is_empty() {
                Some(target.clone())
            } else {
                Some(format!("{target}::{}", tail.join("::")))
            }
        }

        match self {
            Literal::Expr(..) => {}
            Literal::Path {
                associated_to,
                name,
            } => {
                if let Some((path, export_name)) = associated_to {
                    if let Some(resolved) = resolve_alias(path.name(), aliases) {
                        *path = Path::new(resolved.clone());
                        *export_name = resolved;
                    }
                } else if let Some(resolved) = aliases.get(name).cloned() {
                    if let Some((owner, resolved_name)) = resolved.rsplit_once("::") {
                        *associated_to = Some((Path::new(owner.to_owned()), owner.to_owned()));
                        *name = resolved_name.to_owned();
                    } else {
                        *name = resolved;
                    }
                }
            }
            Literal::PostfixUnaryOp { value, .. } => {
                value.resolve_path_head_aliases(aliases);
            }
            Literal::BinOp { left, right, .. } => {
                left.resolve_path_head_aliases(aliases);
                right.resolve_path_head_aliases(aliases);
            }
            Literal::FieldAccess { base, .. } => {
                base.resolve_path_head_aliases(aliases);
            }
            Literal::Struct { fields, .. } => {
                for field in fields.values_mut() {
                    field.value.resolve_path_head_aliases(aliases);
                }
            }
            Literal::Cast { value, .. } => {
                value.resolve_path_head_aliases(aliases);
            }
        }
    }

    fn resolve_module_relative_paths(&mut self, current_module_path: &str) {
        match self {
            Literal::Expr(..) => {}
            Literal::Path {
                associated_to,
                name: _,
            } => {
                if let Some((path, export_name)) = associated_to {
                    if let Some(resolved) =
                        Self::resolve_relative_module_path(path.name(), current_module_path)
                    {
                        if resolved.is_empty() {
                            *associated_to = None;
                        } else {
                            *path = Path::new(resolved.clone());
                            *export_name = resolved;
                        }
                    }
                }
            }
            Literal::PostfixUnaryOp { value, .. } => {
                value.resolve_module_relative_paths(current_module_path);
            }
            Literal::BinOp { left, right, .. } => {
                left.resolve_module_relative_paths(current_module_path);
                right.resolve_module_relative_paths(current_module_path);
            }
            Literal::FieldAccess { base, .. } => {
                base.resolve_module_relative_paths(current_module_path);
            }
            Literal::Struct { fields, .. } => {
                for field in fields.values_mut() {
                    field
                        .value
                        .resolve_module_relative_paths(current_module_path);
                }
            }
            Literal::Cast { value, .. } => {
                value.resolve_module_relative_paths(current_module_path);
            }
        }
    }

    pub fn resolve_dependency_constants(
        &mut self,
        hidden_constants: &HashMap<String, Option<HiddenConstant>>,
        struct_paths: &std::collections::HashSet<Path>,
    ) {
        if let Literal::Path {
            associated_to,
            name,
        } = self
        {
            for hidden_key in Self::hidden_constant_lookup_keys(
                associated_to.as_ref().map(|(path, _)| path),
                name,
                struct_paths,
            ) {
                if let Some(Some(hidden_constant)) = hidden_constants.get(&hidden_key) {
                    *self = hidden_constant.value.clone();
                    self.resolve_dependency_constants(hidden_constants, struct_paths);
                    return;
                }
            }
        }

        match self {
            Literal::Expr(..) | Literal::Path { .. } => {}
            Literal::PostfixUnaryOp { value, .. } => {
                value.resolve_dependency_constants(hidden_constants, struct_paths);
            }
            Literal::BinOp { left, right, .. } => {
                left.resolve_dependency_constants(hidden_constants, struct_paths);
                right.resolve_dependency_constants(hidden_constants, struct_paths);
            }
            Literal::FieldAccess { base, .. } => {
                base.resolve_dependency_constants(hidden_constants, struct_paths);
            }
            Literal::Struct { fields, .. } => {
                for field in fields.values_mut() {
                    field
                        .value
                        .resolve_dependency_constants(hidden_constants, struct_paths);
                }
            }
            Literal::Cast { value, .. } => {
                value.resolve_dependency_constants(hidden_constants, struct_paths);
            }
        }
    }

    pub fn add_dependencies(&self, library: &Library, out: &mut Dependencies) {
        self.visit(&mut |lit| {
            match lit {
                Literal::Struct {
                    ref path,
                    export_name: _,
                    fields: _,
                }
                | Literal::Path {
                    associated_to: Some((ref path, _)),
                    name: _,
                } => {
                    out.add(library, path);
                }
                _ => {}
            }
            true
        });
    }

    pub fn replace_self_with(&mut self, self_ty: &Path) {
        match *self {
            Literal::PostfixUnaryOp { ref mut value, .. } => {
                value.replace_self_with(self_ty);
            }
            Literal::BinOp {
                ref mut left,
                ref mut right,
                ..
            } => {
                left.replace_self_with(self_ty);
                right.replace_self_with(self_ty);
            }
            Literal::FieldAccess { ref mut base, .. } => {
                base.replace_self_with(self_ty);
            }
            Literal::Struct {
                ref mut path,
                ref mut export_name,
                ref mut fields,
            } => {
                if path.replace_self_with(self_ty) {
                    self_ty.name().clone_into(export_name);
                }
                for ref mut expr in fields.values_mut() {
                    expr.value.replace_self_with(self_ty);
                }
            }
            Literal::Cast {
                ref mut ty,
                ref mut value,
            } => {
                ty.replace_self_with(self_ty);
                value.replace_self_with(self_ty);
            }
            Literal::Path {
                ref mut associated_to,
                ..
            } => {
                if let Some((ref mut path, ref mut export_name)) = *associated_to {
                    if path.replace_self_with(self_ty) {
                        self_ty.name().clone_into(export_name);
                    }
                }
            }
            Literal::Expr(..) => {}
        }
    }

    fn is_valid(&self, bindings: &Bindings) -> bool {
        match *self {
            Literal::Expr(..) => true,
            Literal::Path {
                ref associated_to,
                ref name,
            } => {
                if let Some((ref path, _export_name)) = associated_to {
                    return bindings.struct_exists(path)
                        || to_known_assoc_constant(path, name).is_some();
                }
                true
            }
            Literal::PostfixUnaryOp { ref value, .. } => value.is_valid(bindings),
            Literal::BinOp {
                ref left,
                ref right,
                ..
            } => left.is_valid(bindings) && right.is_valid(bindings),
            Literal::FieldAccess { ref base, .. } => base.is_valid(bindings),
            Literal::Struct { ref path, .. } => bindings.struct_exists(path),
            Literal::Cast { ref value, .. } => value.is_valid(bindings),
        }
    }

    pub(crate) fn can_be_constexpr(&self) -> bool {
        !self.has_pointer_casts()
    }

    fn visit(&self, visitor: &mut impl FnMut(&Self) -> bool) -> bool {
        if !visitor(self) {
            return false;
        }
        match self {
            Literal::Expr(..) | Literal::Path { .. } => true,
            Literal::PostfixUnaryOp { ref value, .. } => value.visit(visitor),
            Literal::BinOp {
                ref left,
                ref right,
                ..
            } => left.visit(visitor) && right.visit(visitor),
            Literal::FieldAccess { ref base, .. } => base.visit(visitor),
            Literal::Struct { ref fields, .. } => {
                for (_name, field) in fields.iter() {
                    if !field.value.visit(visitor) {
                        return false;
                    }
                }
                true
            }
            Literal::Cast { ref value, .. } => value.visit(visitor),
        }
    }

    fn has_pointer_casts(&self) -> bool {
        let mut has_pointer_casts = false;
        self.visit(&mut |lit| {
            if let Literal::Cast { ref ty, .. } = *lit {
                has_pointer_casts = has_pointer_casts || ty.is_ptr();
            }
            !has_pointer_casts
        });
        has_pointer_casts
    }

    pub fn uses_only_primitive_types(&self) -> bool {
        let mut uses_only_primitive_types = true;
        self.visit(&mut |lit| {
            // XXX This is a bit sketchy, but alas.
            uses_only_primitive_types = uses_only_primitive_types
                && match *lit {
                    Literal::Struct { .. } => false,
                    Literal::Cast { ref ty, .. } => ty.is_primitive_or_ptr_primitive(),
                    _ => true,
                };
            uses_only_primitive_types
        });
        uses_only_primitive_types
    }
}

impl Literal {
    pub fn rename_for_config(&mut self, config: &Config) {
        match self {
            Literal::Struct {
                ref mut export_name,
                fields,
                ..
            } => {
                config.export.rename(export_name);
                for lit in fields.values_mut() {
                    lit.value.rename_for_config(config);
                }
            }
            Literal::FieldAccess { ref mut base, .. } => {
                base.rename_for_config(config);
            }
            Literal::Path {
                ref mut associated_to,
                ref mut name,
            } => {
                if let Some((_path, ref mut export_name)) = associated_to {
                    config.export.rename(export_name);
                } else {
                    config.export.rename(name);
                }
            }
            Literal::PostfixUnaryOp { ref mut value, .. } => {
                value.rename_for_config(config);
            }
            Literal::BinOp {
                ref mut left,
                ref mut right,
                ..
            } => {
                left.rename_for_config(config);
                right.rename_for_config(config);
            }
            Literal::Expr(_) => {}
            Literal::Cast {
                ref mut ty,
                ref mut value,
            } => {
                ty.rename_for_config(config, &GenericParams::default());
                value.rename_for_config(config);
            }
        }
    }

    // Translate from full blown `syn::Expr` into a simpler `Literal` type
    pub fn load(expr: &syn::Expr) -> Result<Literal, String> {
        match *expr {
            // Match binary expressions of the form `a * b`
            syn::Expr::Binary(ref bin_expr) => {
                let l = Self::load(&bin_expr.left)?;
                let r = Self::load(&bin_expr.right)?;
                let op = match bin_expr.op {
                    syn::BinOp::Add(..) => "+",
                    syn::BinOp::Sub(..) => "-",
                    syn::BinOp::Mul(..) => "*",
                    syn::BinOp::Div(..) => "/",
                    syn::BinOp::Rem(..) => "%",
                    syn::BinOp::And(..) => "&&",
                    syn::BinOp::Or(..) => "||",
                    syn::BinOp::BitXor(..) => "^",
                    syn::BinOp::BitAnd(..) => "&",
                    syn::BinOp::BitOr(..) => "|",
                    syn::BinOp::Shl(..) => "<<",
                    syn::BinOp::Shr(..) => ">>",
                    syn::BinOp::Eq(..) => "==",
                    syn::BinOp::Lt(..) => "<",
                    syn::BinOp::Le(..) => "<=",
                    syn::BinOp::Ne(..) => "!=",
                    syn::BinOp::Ge(..) => ">=",
                    syn::BinOp::Gt(..) => ">",
                    syn::BinOp::AddAssign(..) => "+=",
                    syn::BinOp::SubAssign(..) => "-=",
                    syn::BinOp::MulAssign(..) => "*=",
                    syn::BinOp::DivAssign(..) => "/=",
                    syn::BinOp::RemAssign(..) => "%=",
                    syn::BinOp::BitXorAssign(..) => "^=",
                    syn::BinOp::BitAndAssign(..) => "&=",
                    syn::BinOp::BitOrAssign(..) => "|=",
                    syn::BinOp::ShlAssign(..) => "<<=",
                    syn::BinOp::ShrAssign(..) => ">>=",
                    currently_unknown => {
                        return Err(format!(
                            "unsupported binary operator: {currently_unknown:?}"
                        ))
                    }
                };
                Ok(Literal::BinOp {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                })
            }

            // Match literals like true, 'a', 32 etc
            syn::Expr::Lit(syn::ExprLit { ref lit, .. }) => {
                match lit {
                    syn::Lit::Byte(ref value) => Ok(Literal::Expr(format!("{}", value.value()))),
                    syn::Lit::Char(ref value) => Ok(Literal::Expr(match value.value() as u32 {
                        0..=255 => format!("'{}'", value.value().escape_default()),
                        other_code => format!(r"U'\U{other_code:08X}'"),
                    })),
                    syn::Lit::Str(ref value) => Ok(Literal::Expr(escape_c_string_literal(
                        value.value().as_bytes(),
                    )?)),
                    syn::Lit::CStr(ref value) => Ok(Literal::Expr(escape_c_string_literal(
                        &value.value().into_bytes(),
                    )?)),
                    syn::Lit::Int(ref value) => {
                        let suffix = match value.suffix() {
                            "u64" => "ull",
                            "i64" => "ll",
                            "u32" => "u",
                            _ if value.base10_parse::<i64>().is_err() => "ull",
                            _ => "",
                        };
                        Ok(Literal::Expr(format!(
                            "{}{}",
                            value.base10_digits(),
                            suffix
                        )))
                    }
                    syn::Lit::Float(ref value) => {
                        Ok(Literal::Expr(value.base10_digits().to_string()))
                    }
                    syn::Lit::Bool(ref value) => Ok(Literal::Expr(format!("{}", value.value))),
                    // TODO: Add support for byte string and Verbatim
                    _ => Err(format!("Unsupported literal expression. {:?}", *lit)),
                }
            }

            syn::Expr::Field(syn::ExprField {
                ref base,
                ref member,
                ..
            }) => Ok(Literal::FieldAccess {
                base: Box::new(Literal::load(base)?),
                field: member_to_ident(member),
            }),

            syn::Expr::Call(syn::ExprCall {
                ref func, ref args, ..
            }) => {
                let struct_name = match Literal::load(func)? {
                    Literal::Path {
                        associated_to: None,
                        name,
                    } => name,
                    _ => return Err(format!("Unsupported call expression. {:?}", *expr)),
                };
                let mut fields = HashMap::<String, LiteralStructField>::default();
                for (index, arg) in args.iter().enumerate() {
                    let ident =
                        member_to_ident(&syn::Member::Unnamed(syn::Index::from(index))).to_string();
                    let value = Literal::load(arg)?;
                    let field = LiteralStructField { value, cfg: None };
                    fields.insert(ident, field);
                }
                Ok(Literal::Struct {
                    path: Path::new(struct_name.clone()),
                    export_name: struct_name,
                    fields,
                })
            }

            syn::Expr::Struct(syn::ExprStruct {
                ref path,
                ref fields,
                ..
            }) => {
                let struct_name = path.segments[0].ident.unraw().to_string();
                let mut field_map = HashMap::<String, LiteralStructField>::default();
                for field in fields {
                    let ident = member_to_ident(&field.member).to_string();
                    let cfg = Cfg::load(&field.attrs);
                    let value = Literal::load(&field.expr)?;
                    let field = LiteralStructField { value, cfg };
                    field_map.insert(ident, field);
                }
                Ok(Literal::Struct {
                    path: Path::new(struct_name.clone()),
                    export_name: struct_name,
                    fields: field_map,
                })
            }

            syn::Expr::Unary(syn::ExprUnary {
                ref op, ref expr, ..
            }) => match *op {
                UnOp::Not(_) => {
                    let val = Self::load(expr)?;
                    Ok(Literal::PostfixUnaryOp {
                        op: "~",
                        value: Box::new(val),
                    })
                }
                UnOp::Neg(_) => {
                    let val = Self::load(expr)?;
                    Ok(Literal::PostfixUnaryOp {
                        op: "-",
                        value: Box::new(val),
                    })
                }
                _ => Err(format!("Unsupported Unary expression. {:?}", *op)),
            },

            // Match identifiers, like `5 << SHIFT`
            syn::Expr::Path(syn::ExprPath { ref path, .. }) => {
                // Handle only the simplest identifiers and Associated::IDENT
                // kind of syntax.
                Ok(match path.segments.len() {
                    0 => return Err(format!("Unsupported path expression. {path:?}")),
                    1 => Literal::Path {
                        associated_to: None,
                        name: path.segments[0].ident.to_string(),
                    },
                    _ => {
                        let struct_name = path
                            .segments
                            .iter()
                            .take(path.segments.len() - 1)
                            .map(|segment| segment.ident.unraw().to_string())
                            .collect::<Vec<_>>()
                            .join("::");
                        Literal::Path {
                            associated_to: Some((Path::new(&struct_name), struct_name)),
                            name: path.segments.last().unwrap().ident.unraw().to_string(),
                        }
                    }
                })
            }

            syn::Expr::Paren(syn::ExprParen { ref expr, .. }) => Self::load(expr),

            syn::Expr::Cast(syn::ExprCast {
                ref expr, ref ty, ..
            }) => {
                let val = Self::load(expr)?;
                match Type::load(ty)? {
                    Some(ty) => Ok(Literal::Cast {
                        ty,
                        value: Box::new(val),
                    }),
                    None => Err("Cannot cast to zero sized type.".to_owned()),
                }
            }

            _ => Err(format!("Unsupported expression. {:?}", *expr)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub path: Path,
    pub export_name: String,
    pub ty: Type,
    pub value: Literal,
    pub cfg: Option<Cfg>,
    pub annotations: AnnotationSet,
    pub documentation: Documentation,
    pub associated_to: Option<Path>,
}

#[derive(Debug, Clone)]
pub struct HiddenConstant {
    pub value: Literal,
    pub documentation: Documentation,
}

impl Constant {
    pub fn load(
        path: Path,
        mod_cfg: Option<&Cfg>,
        ty: &syn::Type,
        expr: &syn::Expr,
        attrs: &[syn::Attribute],
        associated_to: Option<Path>,
    ) -> Result<Constant, String> {
        let string_constant_ty = string_constant_type(ty);
        let ty = match string_constant_ty {
            Some(ty) => Some(ty),
            None => Type::load(ty)?,
        };
        let mut ty = match ty {
            Some(ty) => ty,
            None => {
                return Err("Cannot have a zero sized const definition.".to_owned());
            }
        };

        let mut lit = Literal::load(expr)?;

        if let Some(ref associated_to) = associated_to {
            ty.replace_self_with(associated_to);
            lit.replace_self_with(associated_to);
        }

        Ok(Constant::new(
            path,
            ty,
            lit,
            Cfg::append(mod_cfg, Cfg::load(attrs)),
            AnnotationSet::load(attrs)?,
            Documentation::load(attrs),
            associated_to,
        ))
    }

    pub fn new(
        path: Path,
        ty: Type,
        value: Literal,
        cfg: Option<Cfg>,
        annotations: AnnotationSet,
        documentation: Documentation,
        associated_to: Option<Path>,
    ) -> Self {
        let export_name = match associated_to.clone() {
            Some(associated_to) => path
                .name()
                .strip_suffix(associated_to.name())
                .unwrap()
                .to_owned(),
            None => path.name().to_owned(),
        };

        Self {
            path,
            export_name,
            ty,
            value,
            cfg,
            annotations,
            documentation,
            associated_to,
        }
    }

    pub fn uses_only_primitive_types(&self) -> bool {
        self.value.uses_only_primitive_types() && self.ty.is_primitive_or_ptr_primitive()
    }
}

impl Item for Constant {
    fn path(&self) -> &Path {
        &self.path
    }

    fn add_dependencies(&self, library: &Library, out: &mut Dependencies) {
        self.ty.add_dependencies(library, out);
        self.value.add_dependencies(library, out);
    }

    fn export_name(&self) -> &str {
        &self.export_name
    }

    fn cfg(&self) -> Option<&Cfg> {
        self.cfg.as_ref()
    }

    fn annotations(&self) -> &AnnotationSet {
        &self.annotations
    }

    fn annotations_mut(&mut self) -> &mut AnnotationSet {
        &mut self.annotations
    }

    fn documentation(&self) -> &Documentation {
        &self.documentation
    }

    fn container(&self) -> ItemContainer {
        ItemContainer::Constant(self.clone())
    }

    fn rename_for_config(&mut self, config: &Config) {
        if self.associated_to.is_none() {
            config.export.rename(&mut self.export_name);
        }
        self.value.rename_for_config(config);
        self.ty.rename_for_config(config, &GenericParams::default()); // FIXME: should probably propagate something here
    }

    fn resolve_declaration_types(&mut self, resolver: &DeclarationTypeResolver) {
        self.ty.resolve_declaration_types(resolver);
    }

    fn generic_params(&self) -> &GenericParams {
        GenericParams::empty()
    }
}

impl Constant {
    fn dependency_constant_key(
        &self,
        struct_paths: &std::collections::HashSet<Path>,
    ) -> Option<String> {
        match &self.value {
            Literal::Path {
                associated_to,
                name,
            } => Literal::hidden_constant_lookup_keys(
                associated_to.as_ref().map(|(path, _)| path),
                name,
                struct_paths,
            )
            .into_iter()
            .next(),
            _ => None,
        }
    }

    pub fn resolve_path_aliases(&mut self, aliases: &HashMap<String, String>) {
        self.value.resolve_path_head_aliases(aliases);
    }

    pub fn resolve_module_relative_paths(&mut self, current_module_path: &str) {
        self.value
            .resolve_module_relative_paths(current_module_path);
    }

    pub fn resolve_dependency_constants(
        &mut self,
        hidden_constants: &HashMap<String, Option<HiddenConstant>>,
        struct_paths: &std::collections::HashSet<Path>,
    ) {
        if self.documentation.is_empty() {
            if let Some(key) = self.dependency_constant_key(struct_paths) {
                if let Some(Some(hidden_constant)) = hidden_constants.get(&key) {
                    self.documentation = hidden_constant.documentation.clone();
                }
            }
        }
        self.value
            .resolve_dependency_constants(hidden_constants, struct_paths);
    }

    fn should_write_as_string_macro(&self) -> bool {
        matches!(
            (&self.ty, &self.value),
            (
                Type::Ptr {
                    ty,
                    is_const: true,
                    ..
                },
                Literal::Expr(value)
            ) if matches!(ty.as_ref(), Type::Primitive(PrimitiveType::Char)) && value.starts_with('"')
        )
    }

    pub fn write_declaration<F: Write, LB: LanguageBackend>(
        &self,
        config: &Config,
        language_backend: &mut LB,
        out: &mut SourceWriter<F>,
        associated_to_struct: &Struct,
    ) {
        debug_assert!(self.associated_to.is_some());
        debug_assert!(config.language == Language::Cxx);
        debug_assert!(!associated_to_struct.is_transparent);
        debug_assert!(config.structure.associated_constants_in_body);
        debug_assert!(config.constant.allow_static_const);

        let condition = self.cfg.to_condition(config);
        condition.write_before(config, out);
        if let Type::Ptr { is_const: true, .. } = self.ty {
            out.write("static ");
        } else {
            out.write("static const ");
        }
        language_backend.write_type(out, &self.ty);
        write!(out, " {};", self.export_name());
        condition.write_after(config, out);
    }

    pub fn write<F: Write, LB: LanguageBackend>(
        &self,
        config: &Config,
        language_backend: &mut LB,
        out: &mut SourceWriter<F>,
        associated_to_struct: Option<&Struct>,
    ) {
        if let Some(assoc) = associated_to_struct {
            if assoc.is_generic() {
                return; // Not tested / implemented yet, so bail out.
            }
        }

        if !self.value.is_valid(out.bindings()) {
            return;
        }

        let write_as_string_macro = self.should_write_as_string_macro();

        let associated_to_transparent = associated_to_struct.is_some_and(|s| s.is_transparent);

        let in_body = !write_as_string_macro
            && associated_to_struct.is_some()
            && config.language == Language::Cxx
            && config.structure.associated_constants_in_body
            && config.constant.allow_static_const
            && !associated_to_transparent;

        let condition = self.cfg.to_condition(config);
        condition.write_before(config, out);

        let name = if in_body {
            Cow::Owned(format!(
                "{}::{}",
                associated_to_struct.unwrap().export_name(),
                self.export_name(),
            ))
        } else if self.associated_to.is_none() {
            Cow::Borrowed(self.export_name())
        } else {
            let associated_name = match associated_to_struct {
                Some(s) => {
                    let name = s.export_name();
                    let rules = s
                        .annotations
                        .parse_atom::<RenameRule>("rename-associated-constant");
                    let rules = rules
                        .as_ref()
                        .unwrap_or(&config.structure.rename_associated_constant);

                    if let Some(r) = rules.not_none() {
                        r.apply(name, IdentifierType::Type)
                    } else {
                        Cow::Borrowed(name)
                    }
                }
                None => {
                    let mut name = self.associated_to.as_ref().unwrap().name().to_owned();
                    config.export.rename(&mut name);
                    Cow::Owned(name)
                }
            };

            Cow::Owned(format!("{}_{}", associated_name, self.export_name()))
        };

        let mut value = &self.value;
        while let Literal::Struct { path, fields, .. } = value {
            if !out.bindings().struct_is_transparent(path) {
                break;
            }
            value = &fields.iter().next().unwrap().1.value
        }

        language_backend.write_documentation(out, self.documentation());

        let allow_constexpr = !write_as_string_macro
            && config.constant.allow_constexpr
            && self.value.can_be_constexpr();
        match config.language {
            Language::Cxx
                if !write_as_string_macro
                    && (config.constant.allow_static_const || allow_constexpr) =>
            {
                if allow_constexpr {
                    out.write("constexpr ")
                }

                if config.constant.allow_static_const {
                    out.write(if in_body { "inline " } else { "static " });
                }

                if let Type::Ptr { is_const: true, .. } = self.ty {
                    // Nothing.
                } else {
                    out.write("const ");
                }

                language_backend.write_type(out, &self.ty);
                write!(out, " {name} = ");
                language_backend.write_literal(out, value);
                write!(out, ";");
            }
            Language::Cxx | Language::C => {
                write!(out, "#define {name} ");
                language_backend.write_literal(out, value);
            }
            Language::Cython => {
                if let Type::Ptr { is_const: true, .. } = self.ty {
                } else {
                    out.write("const ");
                }
                language_backend.write_type(out, &self.ty);
                // For extern Cython declarations the initializer is ignored,
                // but still useful as documentation, so we write it as a comment.
                write!(out, " {name} # = ");
                language_backend.write_literal(out, value);
            }
        }

        condition.write_after(config, out);
    }
}
