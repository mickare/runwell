// Copyright 2019 Robin Freyler
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::parse::{
    FunctionBody,
    FunctionId,
    FunctionSigId,
    Module,
    ModuleBuilder,
    ParseError,
};
use wasmparser::{
    CodeSectionReader,
    DataSectionReader,
    ElementSectionReader,
    ExportSectionReader,
    FunctionSectionReader,
    GlobalSectionReader,
    ImportSectionEntryType,
    ImportSectionReader,
    MemorySectionReader,
    ModuleReader,
    OperatorValidatorConfig,
    Section,
    SectionCode,
    TableSectionReader,
    TypeSectionReader,
    ValidatingParserConfig,
};

/// The internals of the parser.
pub struct ParserInternals<'a> {
    /// The module to contain the parsed Wasm module.
    module: ModuleBuilder<'a>,
    /// The underlying Wasm file reader.
    reader: ModuleReader<'a>,
    /// The last encountered section.
    section: Section<'a>,
}

impl<'a> ParserInternals<'a> {
    /// Creates a new Wasm parser.
    fn new(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let mut reader = ModuleReader::new(bytes)?;
        Ok(Self {
            module: Module::build(),
            section: reader.read()?,
            reader,
        })
    }
}

/// The parser and the states it can be in.
pub enum Parser<'a> {
    /// The parser is running and has not encountered errors.
    Parsing(Box<ParserInternals<'a>>),
    /// The parser reached the end of the Wasm file and is done.
    Done(Box<Module<'a>>),
    /// The parser encountered an error.
    Error(ParseError),
}

impl<'a> Parser<'a> {
    /// Creates a new parser from the given bytes.
    fn new(bytes: &'a [u8]) -> Self {
        match ParserInternals::new(bytes) {
            Ok(parser) => Parser::Parsing(Box::new(parser)),
            Err(error) => Parser::Error(error),
        }
    }

    /// Applies `f` for parsing the given Wasm section.
    ///
    /// # Dev Note
    ///
    /// This is the heart of the parser with a mondaic interface.
    /// It will simply apply `f` if the current section of the parser
    /// is the one that applies to `code` and otherwise does nothing.
    ///
    /// Encountering errors will yield in `Parser::Error` state.
    /// Encountering the end of the file will yield `Parser::EndOfFile` state.
    fn for_section<F>(self, code: SectionCode, f: F) -> Self
    where
        F: FnOnce(
            &Section<'a>,
            &mut ModuleBuilder<'a>,
        ) -> Result<(), ParseError>,
    {
        match self {
            Parser::Parsing(mut parser) if parser.section.code == code => {
                if let Err(error) = f(&parser.section, &mut parser.module) {
                    return Parser::Error(error)
                }
                // TODO: Maybe insert another check for `reader.eof` here.
                if let Err(error) = parser.reader.skip_custom_sections() {
                    return Parser::Error(error.into())
                }
                if parser.reader.eof() {
                    return Parser::Done(Box::new(parser.module.finalize()))
                }
                match parser.reader.read() {
                    Err(error) => Parser::Error(error.into()),
                    Ok(section) => {
                        parser.section = section;
                        Parser::Parsing(parser)
                    }
                }
            }
            otherwise => otherwise,
        }
    }

    /// Finalizes parsing and returns the resulting module or an error.
    fn finish(self) -> Result<Module<'a>, ParseError> {
        match self {
            Parser::Parsing(parser) => Ok(parser.module.finalize()),
            Parser::Done(module) => Ok(*module),
            Parser::Error(error) => Err(error),
        }
    }
}

/// Parses a byte stream representing a binary Wasm module.
///
/// # Dev Note
///
/// - For the sake of simplicity we ignore custom sections.
/// - We have to skip custom section after every step
///   since they might appear out of order.
/// - The binary Wasm sections are guaranteed to be in the following order.
///   Sections are optional and may be missing.
///
/// | Section  | Description                              |
/// |----------|------------------------------------------|
/// | Type     | Function signature declarations |
/// | Import   | Import declarations |
/// | Function | Function declarations |
/// | Table    | Indirect function table and other tables |
/// | Memory   | Memory attributes |
/// | Global   | Global declarations |
/// | Export   | Exports |
/// | Start    | Start function declaration |
/// | Element  | Elements section |
/// | Code     | Function bodies (code) |
/// | Data     | Data segments |
pub fn parse(bytes: &[u8]) -> Result<Module, ParseError> {
    validate_wasm(bytes)?;
    use SectionCode::*;
    Parser::new(bytes)
        .for_section(Type, |section, module| {
            parse_types(section.get_type_section_reader()?, module)
        })
        .for_section(Import, |section, module| {
            parse_imports(section.get_import_section_reader()?, module)
        })
        .for_section(Function, |section, module| {
            parse_function_signatures(
                section.get_function_section_reader()?,
                module,
            )
        })
        .for_section(Table, |section, module| {
            parse_tables(section.get_table_section_reader()?, module)
        })
        .for_section(Memory, |section, module| {
            parse_linear_memories(section.get_memory_section_reader()?, module)
        })
        .for_section(Global, |section, module| {
            parse_globals(section.get_global_section_reader()?, module)
        })
        .for_section(Export, |section, module| {
            parse_exports(section.get_export_section_reader()?, module)
        })
        .for_section(Start, |section, module| {
            parse_start(section.get_start_section_content()?, module)
        })
        .for_section(Element, |section, module| {
            parse_element(section.get_element_section_reader()?, module)
        })
        .for_section(Code, |section, module| {
            parse_code(section.get_code_section_reader()?, module)
        })
        .for_section(Data, |section, module| {
            parse_data(section.get_data_section_reader()?, module)
        })
        .finish()
}

/// Validates the Wasm bytes for the `runwell` JIT compiler.
///
/// # Notes
///
/// | Config                   | flag    | Note                               |
/// |:-------------------------|:-------:|:-----------------------------------|
/// | `enable_threads`         | `false` | Not useful for blockchain.         |
/// | `enable_reference_types` | `false` | Config might change in the future. |
/// | `enable_simd`            | `false` | Not useful for blockchain.         |
/// | `enable_bulk_memory`     | `false` | Not useful for blockchain.         |
/// | `enable_multi_value`     | `false` | Config might change in the future. |
/// | `deterministic_only`     | `true`  | Disables floating points.          |
fn validate_wasm(bytes: &[u8]) -> Result<(), ParseError> {
    wasmparser::validate(
        bytes,
        Some(ValidatingParserConfig {
            operator_config: OperatorValidatorConfig {
                enable_threads: false,
                enable_reference_types: false,
                enable_simd: false,
                enable_bulk_memory: false,
                enable_multi_value: false,
                deterministic_only: true,
            },
        }),
    )?;
    Ok(())
}

fn parse_types<'a>(
    reader: TypeSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for signature in reader.into_iter() {
        module.push_fn_signature(signature?.into());
    }
    Ok(())
}

fn parse_imports<'a>(
    reader: ImportSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for import in reader.into_iter() {
        let import = import?;
        let module_name = import.module;
        let field_name = import.field;
        match import.ty {
            ImportSectionEntryType::Function(fn_sig_id) => {
                module.push_imported_fn(
                    module_name,
                    field_name,
                    FunctionSigId(fn_sig_id as usize),
                )?
            }
            ImportSectionEntryType::Table(table_type) => {
                module.push_imported_table(
                    module_name,
                    field_name,
                    table_type,
                )?;
            }
            ImportSectionEntryType::Memory(memory_type) => {
                module.push_imported_linear_memory(
                    module_name,
                    field_name,
                    memory_type,
                )?;
            }
            ImportSectionEntryType::Global(global_type) => {
                module.push_imported_global(
                    module_name,
                    field_name,
                    global_type.into(),
                )?;
            }
        }
    }
    Ok(())
}

fn parse_function_signatures<'a>(
    reader: FunctionSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for fn_sig in reader.into_iter() {
        module.push_internal_fn(FunctionSigId(fn_sig? as usize))
    }
    Ok(())
}

fn parse_tables<'a>(
    reader: TableSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for table_type in reader.into_iter() {
        module.push_internal_table(table_type?)
    }
    Ok(())
}

fn parse_linear_memories<'a>(
    reader: MemorySectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for memory_type in reader.into_iter() {
        module.push_internal_linear_memory(memory_type?)
    }
    Ok(())
}

fn parse_globals<'a>(
    reader: GlobalSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for global_type in reader.into_iter() {
        use core::convert::TryInto;
        let global_type = global_type?;
        module.push_internal_global(global_type.ty.into());
        module.push_global_initializer(global_type.init_expr.try_into()?);
    }
    Ok(())
}

fn parse_exports<'a>(
    reader: ExportSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for export in reader.into_iter() {
        module.push_export(export?.into());
    }
    Ok(())
}

fn parse_start<'a>(
    start_fn_id: u32,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    module.set_start_fn(FunctionId(start_fn_id as usize));
    Ok(())
}

fn parse_element<'a>(
    _reader: ElementSectionReader<'a>,
    _module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    Ok(())
}

fn parse_code<'a>(
    reader: CodeSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for function_body in reader.into_iter() {
        let function_body = function_body?;
        let locals = function_body
            .get_locals_reader()?
            .into_iter()
            .map(|local| {
                match local {
                    Ok((num, ty)) => Ok((num as usize, ty)),
                    Err(err) => Err(err),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ops = function_body
            .get_operators_reader()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        module.push_fn_body(FunctionBody::new(locals, ops));
    }
    Ok(())
}

fn parse_data<'a>(
    reader: DataSectionReader<'a>,
    module: &mut ModuleBuilder<'a>,
) -> Result<(), ParseError> {
    for data in reader.into_iter() {
        module.push_data(data?)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_incrementer() {
        let wasm = include_bytes!("../../incrementer.wasm");
        let module = parse(wasm).expect("invalid Wasm byte code");
        // println!("{:#?}", module);
        // let module = Module::new(wasm).expect("couldn't parse Wasm module");
        for fun in module.iter_internal_fns().skip(110).take(10) {
            println!("\n\n{:#?}", fun);
        }
        // for fun in module.iter_fns().take(2) {
        //     println!("\n\n{:#?}", fun);
        // }
        for global_variable in module.iter_internal_globals().take(2) {
            println!("\n{:#?}", global_variable)
        }
        for export in module.iter_exports().take(2) {
            println!("\n{:#?}", export);
        }
        // println!("\nmemory = {:#?}", module.memory);
        // println!("\nstart fn           = {:#?}", module.start_fn());
        // println!("# imported fns     = {:#?}", module.len_imported_fns());
        // println!("# fns              = {:#?}", module.len_fns());
        // println!("# imported globals = {:#?}", module.len_imported_globals());
        // println!("# globals          = {:#?}", module.len_globals());
    }
}
