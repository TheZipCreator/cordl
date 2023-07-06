use pathdiff::diff_paths;

use crate::STATIC_CONFIG;

use super::{context::CppContext, cpp_type::CppType};
use std::path::{Path, PathBuf};
use std::io::Write;
use color_eyre::eyre::eyre;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Default, PartialOrd, Ord)]
pub struct CppTemplate {
    pub names: Vec<String>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Default, PartialOrd, Ord)]
pub struct CppStructSpecialization {
    pub name: String,
    pub namespace: Option<String>,
    pub is_struct: bool,
    pub template: CppTemplate,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct CppForwardDeclareGroup {
    // TODO: Make this group lots into a single namespace
    pub namespace: Option<String>,
    pub items: Vec<CppForwardDeclare>,
    pub group_items: Vec<CppForwardDeclareGroup>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct CppForwardDeclare {
    // TODO: Make this group lots into a single namespace
    pub is_struct: bool,
    pub namespace: Option<String>,
    pub name: String,
    pub templates: Option<CppTemplate>, // names of template arguments, T, TArgs etc.
    pub literals: Option<Vec<String>>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct CppCommentedString {
    pub data: String,
    pub comment: Option<String>,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CppInclude {
    pub include: PathBuf,
    pub system: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CppUsingAlias {
    pub result: String,
    pub result_literals: Vec<String>,

    pub alias: String,
    pub namespaze: Option<String>,
    pub template: Option<CppTemplate>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CppMember {
    FieldDecl(CppFieldDecl),
    FieldImpl(CppFieldImpl),
    MethodDecl(CppMethodDecl),
    MethodImpl(CppMethodImpl),
    Property(CppProperty),
    Comment(CppCommentedString),
    ConstructorDecl(CppConstructorDecl),
    ConstructorImpl(CppConstructorImpl),
    CppUsingAlias(CppUsingAlias)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppMethodData {
    pub estimated_size: usize,
    pub addrs: u64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppMethodSizeStruct {
    pub cpp_method_name: String,
    pub complete_type_name: String,
    pub ret_ty: String,
    pub instance: bool,
    pub params: Vec<CppParam>,
    pub method_data: CppMethodData,

    pub template: CppTemplate,

    pub interface_clazz_of: String,
    pub is_final: bool,
    pub slot: Option<u16>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppFieldDecl {
    pub name: String,
    pub ty: String,
    pub offset: u32,
    pub instance: bool,
    pub readonly: bool,
    pub classof_call: String,
    pub literal_value: Option<String>,
    pub is_value_type: bool,
    pub declaring_is_reference: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppFieldImpl {
    pub klass_name: String,
    pub field: CppFieldDecl,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppParam {
    pub name: String,
    pub ty: String,
    // TODO: Use bitflags to indicate these attributes
    // May hold:
    // const
    // May hold one of:
    // *
    // &
    // &&
    pub modifiers: String,
    pub def_value: Option<String>,
}

// TODO: Generics
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppMethodDecl {
    pub cpp_name: String,
    pub return_type: String,
    pub parameters: Vec<CppParam>,
    pub instance: bool,
    pub template: CppTemplate,
    // TODO: Use bitflags to indicate these attributes
    // Holds unique of:
    // const
    // override
    // noexcept
    pub suffix_modifiers: String,
    // Holds unique of:
    // constexpr
    // static
    // inline
    // explicit(...)
    // virtual
    pub prefix_modifiers: String,
    // TODO: Add all descriptions missing for the method
    pub method_data: Option<CppMethodData>,
    pub is_virtual: bool,
}

// TODO: Generic
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppMethodImpl {
    pub cpp_method_name: String,
    pub cs_method_name: String,

    pub holder_cpp_namespaze: String,
    pub holder_cpp_name: String,

    pub return_type: String,
    pub parameters: Vec<CppParam>,
    pub instance: bool,

    pub template: CppTemplate,
    // TODO: Use bitflags to indicate these attributes
    // Holds unique of:
    // const
    // override
    // noexcept
    pub suffix_modifiers: String,
    // Holds unique of:
    // constexpr
    // static
    // inline
    // explicit(...)
    // virtual
    pub prefix_modifiers: String,
}

// TODO: Generics
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppConstructorDecl {
    pub ty: String,
    pub parameters: Vec<CppParam>,
    pub template: CppTemplate,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppConstructorImpl {
    pub holder_cpp_ty_name: String,

    pub parameters: Vec<CppParam>,
    pub is_constexpr: bool,
    pub template: CppTemplate,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CppProperty {
    pub name: String,
    pub ty: String,
    pub setter: Option<CppMethodData>,
    pub getter: Option<CppMethodData>,
    pub abstr: bool,
    pub instance: bool,
    pub classof_call: String,
}
// Writing

impl CppForwardDeclare {
    pub fn from_cpp_type(cpp_type: &CppType) -> Self {
        let ns = if cpp_type.nested {
            None
        } else {
            Some(cpp_type.cpp_namespace().to_string())
        };

        Self {
            is_struct: cpp_type.is_value_type,
            namespace: ns,
            name: cpp_type.name().clone(),
            templates: cpp_type.cpp_template.clone(),
            literals: cpp_type.generic_instantiation_args.clone(),
        }
    }
}

impl CppParam {
    pub fn params_as_args(params: &[CppParam]) -> impl Iterator<Item = String> + '_ {
        params.iter().map(|p| match &p.def_value {
            Some(val) => format!("{}{} {} = {val}", p.ty, p.modifiers, p.name),
            None => format!("{}{} {}", p.ty, p.modifiers, p.name),
        })
    }
    pub fn params_as_args_no_default(params: &[CppParam]) -> impl Iterator<Item = String> + '_ {
        params
            .iter()
            .map(|p| format!("{}{} {}", p.ty, p.modifiers, p.name))
    }
    pub fn params_names(params: &[CppParam]) -> impl Iterator<Item = &String> {
        params.iter().map(|p| &p.name)
    }
    pub fn params_types(params: &[CppParam]) -> impl Iterator<Item = &String> {
        params.iter().map(|p| &p.ty)
    }

    pub fn params_il2cpp_types(params: &[CppParam]) -> impl Iterator<Item = String> + '_ {
        params
            .iter()
            .map(|p| format!("::il2cpp_utils::ExtractType({})", p.name))
    }
}

impl CppInclude {
    // smelly use of config but whatever
    pub fn new_context(context: &CppContext) -> Self {
        Self {
            include: diff_paths(&context.typedef_path, &STATIC_CONFIG.header_path).unwrap(),
            system: false,
        }
    }

    pub fn new_system<P: AsRef<Path>>(str: P) -> Self {
        Self {
            include: str.as_ref().to_path_buf(),
            system: true,
        }
    }

    pub fn new<P: AsRef<Path>>(str: P) -> Self {
        Self {
            include: str.as_ref().to_path_buf(),
            system: false,
        }
    }
}

impl CppFieldDecl { // field declaration
    pub fn write_field_getter(&self, writer: &mut super::writer::CppWriter) -> color_eyre::Result<()> {
        let name = &self.name;
        let ty = &self.ty;

        writeln!(writer, "/// @brief Field getter for {name}")?;
        if self.instance { writeln!(writer, "/// @return value at offset 0x{:x}", self.offset)? }

        let static_keyword = if self.instance { "" } else { "static " };
        if self.declaring_is_reference && self.is_value_type {
            let const_keyword = if self.readonly { "const " } else { "" };
            writeln!(writer, "{static_keyword}{const_keyword}{ty}& __get_{name}();")?;
        } else {
            writeln!(writer, "{static_keyword}{ty} __get_{name}();")?;
        }

        Ok(())
    }

    pub fn write_field_putter(&self, writer: &mut super::writer::CppWriter) -> color_eyre::Result<()> {
        if self.readonly {
            return Err(eyre!("can't write putter for readonly field"));
        }

        let name = &self.name;
        let ty = &self.ty;

        writeln!(writer, "/// @brief Field putter for {name}")?;
        writeln!(writer, "/// @param value what to put at offset 0x{:x}", self.offset)?;

        let static_keyword = if self.instance { "" } else { "static " };
        if self.declaring_is_reference && self.is_value_type {
            writeln!(writer, "{static_keyword}void __put_{name}(const {ty}& value);")?;
        } else {
            writeln!(writer, "{static_keyword}void __put_{name}({ty} value);")?;
        }

        Ok(())
    }
}

impl CppFieldImpl {
    pub fn write_field_getter(&self, writer: &mut super::writer::CppWriter)  -> color_eyre::Result<()> {
        let field = &self.field;
        let name = &field.name;
        let ty = &field.ty;
        let klass_name = &self.klass_name;

        if field.declaring_is_reference && field.is_value_type {
            writeln!(writer, "{}{ty}& {klass_name}::__get_{name}() {{", if field.readonly {"const "} else { "" })?;
        } else {
            writeln!(writer, "{ty} {klass_name}::__get_{name}() {{")?;
        }

        if field.is_value_type {
            match field.instance {
                true => writeln!(writer, "return getValueTypeInstance<{ty}, 0x{:x}>({});", field.offset, if field.declaring_is_reference { "instance" } else { "this" })?,
                false => writeln!(writer, "return getValueTypeStatic<{ty}, \"{}\", {}>();", name, field.classof_call)?,
            }
        } else {
            match field.instance {
                true => writeln!(writer, "return getReferenceTypeInstance<{ty}, 0x{:x}>({});", field.offset, if field.declaring_is_reference { "instance" } else { "this" })?,
                false => writeln!(writer, "return getReferenceTypeStatic<{ty}, \"{}\", {}>();", name, field.classof_call)?,
            }
        }

        writeln!(writer, "}}")?;

        Ok(())
    }

    pub fn write_field_putter(&self, writer: &mut super::writer::CppWriter) -> color_eyre::Result<()> {
        if self.field.readonly {
            return Err(eyre!("can't write putter for readonly field"));
        }

        let field = &self.field;
        let name = &field.name;
        let ty = &field.ty;
        let klass_name = &self.klass_name;

        if field.declaring_is_reference && field.is_value_type {
            writeln!(writer, "void {klass_name}::__put_{name}(const {ty}& value) {{")?;
        } else {
            writeln!(writer, "void {klass_name}::__put_{name}({ty} value) {{")?;
        }

        if field.is_value_type {
            match field.instance {
                true => writeln!(writer, "setValueTypeInstance<{ty}, 0x{:x}>({}, value);", field.offset, if field.declaring_is_reference { "instance" } else { "this" })?,
                false => writeln!(writer, "setValueTypeStatic<{ty}, \"{}\", {}>(value);", field.name, field.classof_call)?,
            }
        } else {
            match field.instance {
                true => writeln!(writer, "setReferenceTypeInstance<{ty}, 0x{:x}>({}, value);", field.offset, if field.declaring_is_reference { "instance" } else { "this" })?,
                false => writeln!(writer, "setReferenceTypeStatic<{ty}, \"{}\", {}>(value);", field.name, field.classof_call)?,
            }
        }

        writeln!(writer, "}}")?;

        Ok(())
    }
}
