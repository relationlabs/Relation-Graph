use std::io;
use std::io::{Cursor, Read};
use std::mem::size_of;

use crate::error::invalid_data_error;
use crate::model::xsd::*;
use crate::store::small_string::SmallString;
use crate::StrHash;

pub(crate) type EncodedTerm = crate::store::numeric_encoder::EncodedTerm<StrHash>;
pub(crate) type EncodedQuad = crate::store::numeric_encoder::EncodedQuad<StrHash>;

const WRITTEN_TERM_MAX_SIZE: usize = size_of::<u8>() + 2 * size_of::<StrHash>();

// Encoded term type blocks
// 1-7: usual named nodes (except prefixes c.f. later)
// 8-15: blank nodes
// 16-47: literals
// 48-64: future use
// 64-127: default named node prefixes
// 128-255: custom named node prefixes
const TYPE_NAMED_NODE_ID: u8 = 1;
const TYPE_NUMERICAL_BLANK_NODE_ID: u8 = 8;
const TYPE_SMALL_BLANK_NODE_ID: u8 = 9;
const TYPE_BIG_BLANK_NODE_ID: u8 = 10;
const TYPE_SMALL_STRING_LITERAL: u8 = 16;
const TYPE_BIG_STRING_LITERAL: u8 = 17;
const TYPE_SMALL_SMALL_LANG_STRING_LITERAL: u8 = 20;
const TYPE_SMALL_BIG_LANG_STRING_LITERAL: u8 = 21;
const TYPE_BIG_SMALL_LANG_STRING_LITERAL: u8 = 22;
const TYPE_BIG_BIG_LANG_STRING_LITERAL: u8 = 23;
const TYPE_SMALL_TYPED_LITERAL: u8 = 24;
const TYPE_BIG_TYPED_LITERAL: u8 = 25;
const TYPE_BOOLEAN_LITERAL_TRUE: u8 = 28;
const TYPE_BOOLEAN_LITERAL_FALSE: u8 = 29;
const TYPE_FLOAT_LITERAL: u8 = 30;
const TYPE_DOUBLE_LITERAL: u8 = 31;
const TYPE_INTEGER_LITERAL: u8 = 32;
const TYPE_DECIMAL_LITERAL: u8 = 33;
const TYPE_DATE_TIME_LITERAL: u8 = 34;
const TYPE_TIME_LITERAL: u8 = 35;
const TYPE_DATE_LITERAL: u8 = 36;
const TYPE_G_YEAR_MONTH_LITERAL: u8 = 37;
const TYPE_G_YEAR_LITERAL: u8 = 38;
const TYPE_G_MONTH_DAY_LITERAL: u8 = 39;
const TYPE_G_DAY_LITERAL: u8 = 40;
const TYPE_G_MONTH_LITERAL: u8 = 41;
const TYPE_DURATION_LITERAL: u8 = 42;
const TYPE_YEAR_MONTH_DURATION_LITERAL: u8 = 43;
const TYPE_DAY_TIME_DURATION_LITERAL: u8 = 44;


impl EncodedTerm {
    pub fn to_bytes(self) -> Vec<u8> {
        encode_term(self)
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        // TODO: handle unwrap
        decode_term(data).unwrap()
    }
}

pub fn encode_term(t: EncodedTerm) -> Vec<u8> {
    let mut vec = Vec::with_capacity(WRITTEN_TERM_MAX_SIZE);
    write_term(&mut vec, t);
    vec
}

pub fn encode_term_pair(t1: EncodedTerm, t2: EncodedTerm) -> Vec<u8> {
    let mut vec = Vec::with_capacity(2 * WRITTEN_TERM_MAX_SIZE);
    write_term(&mut vec, t1);
    write_term(&mut vec, t2);
    vec
}

pub fn encode_term_triple(t1: EncodedTerm, t2: EncodedTerm, t3: EncodedTerm) -> Vec<u8> {
    let mut vec = Vec::with_capacity(3 * WRITTEN_TERM_MAX_SIZE);
    write_term(&mut vec, t1);
    write_term(&mut vec, t2);
    write_term(&mut vec, t3);
    vec
}

pub fn encode_term_quad(t1: EncodedTerm, t2: EncodedTerm, t3: EncodedTerm, t4: EncodedTerm) -> Vec<u8> {
    let mut vec = Vec::with_capacity(4 * WRITTEN_TERM_MAX_SIZE);
    write_term(&mut vec, t1);
    write_term(&mut vec, t2);
    write_term(&mut vec, t3);
    write_term(&mut vec, t4);
    vec
}

fn write_term(sink: &mut Vec<u8>, term: EncodedTerm) {
    match term {
        EncodedTerm::DefaultGraph => (),
        EncodedTerm::NamedNode { iri_id } => {
            sink.push(TYPE_NAMED_NODE_ID);
            sink.extend_from_slice(&iri_id.to_be_bytes());
        }
        EncodedTerm::NumericalBlankNode { id } => {
            sink.push(TYPE_NUMERICAL_BLANK_NODE_ID);
            sink.extend_from_slice(&id.to_be_bytes())
        }
        EncodedTerm::SmallBlankNode(id) => {
            sink.push(TYPE_SMALL_BLANK_NODE_ID);
            sink.extend_from_slice(&id.to_be_bytes())
        }
        EncodedTerm::BigBlankNode { id_id } => {
            sink.push(TYPE_BIG_BLANK_NODE_ID);
            sink.extend_from_slice(&id_id.to_be_bytes());
        }
        EncodedTerm::SmallStringLiteral(value) => {
            sink.push(TYPE_SMALL_STRING_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::BigStringLiteral { value_id } => {
            sink.push(TYPE_BIG_STRING_LITERAL);
            sink.extend_from_slice(&value_id.to_be_bytes());
        }
        EncodedTerm::SmallSmallLangStringLiteral { value, language } => {
            sink.push(TYPE_SMALL_SMALL_LANG_STRING_LITERAL);
            sink.extend_from_slice(&language.to_be_bytes());
            sink.extend_from_slice(&value.to_be_bytes());
        }
        EncodedTerm::SmallBigLangStringLiteral { value, language_id } => {
            sink.push(TYPE_SMALL_BIG_LANG_STRING_LITERAL);
            sink.extend_from_slice(&language_id.to_be_bytes());
            sink.extend_from_slice(&value.to_be_bytes());
        }
        EncodedTerm::BigSmallLangStringLiteral { value_id, language } => {
            sink.push(TYPE_BIG_SMALL_LANG_STRING_LITERAL);
            sink.extend_from_slice(&language.to_be_bytes());
            sink.extend_from_slice(&value_id.to_be_bytes());
        }
        EncodedTerm::BigBigLangStringLiteral {
            value_id,
            language_id,
        } => {
            sink.push(TYPE_BIG_BIG_LANG_STRING_LITERAL);
            sink.extend_from_slice(&language_id.to_be_bytes());
            sink.extend_from_slice(&value_id.to_be_bytes());
        }
        EncodedTerm::SmallTypedLiteral { value, datatype_id } => {
            sink.push(TYPE_SMALL_TYPED_LITERAL);
            sink.extend_from_slice(&datatype_id.to_be_bytes());
            sink.extend_from_slice(&value.to_be_bytes());
        }
        EncodedTerm::BigTypedLiteral {
            value_id,
            datatype_id,
        } => {
            sink.push(TYPE_BIG_TYPED_LITERAL);
            sink.extend_from_slice(&datatype_id.to_be_bytes());
            sink.extend_from_slice(&value_id.to_be_bytes());
        }
        EncodedTerm::BooleanLiteral(true) => sink.push(TYPE_BOOLEAN_LITERAL_TRUE),
        EncodedTerm::BooleanLiteral(false) => sink.push(TYPE_BOOLEAN_LITERAL_FALSE),
        EncodedTerm::FloatLiteral(value) => {
            sink.push(TYPE_FLOAT_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DoubleLiteral(value) => {
            sink.push(TYPE_DOUBLE_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::IntegerLiteral(value) => {
            sink.push(TYPE_INTEGER_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DecimalLiteral(value) => {
            sink.push(TYPE_DECIMAL_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DateTimeLiteral(value) => {
            sink.push(TYPE_DATE_TIME_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::TimeLiteral(value) => {
            sink.push(TYPE_TIME_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DurationLiteral(value) => {
            sink.push(TYPE_DURATION_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DateLiteral(value) => {
            sink.push(TYPE_DATE_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::GYearMonthLiteral(value) => {
            sink.push(TYPE_G_YEAR_MONTH_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::GYearLiteral(value) => {
            sink.push(TYPE_G_YEAR_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::GMonthDayLiteral(value) => {
            sink.push(TYPE_G_MONTH_DAY_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::GDayLiteral(value) => {
            sink.push(TYPE_G_DAY_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::GMonthLiteral(value) => {
            sink.push(TYPE_G_MONTH_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::YearMonthDurationLiteral(value) => {
            sink.push(TYPE_YEAR_MONTH_DURATION_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
        EncodedTerm::DayTimeDurationLiteral(value) => {
            sink.push(TYPE_DAY_TIME_DURATION_LITERAL);
            sink.extend_from_slice(&value.to_be_bytes())
        }
    }
}

pub fn decode_term(buffer: &[u8]) -> Result<EncodedTerm, io::Error> {
    Cursor::new(&buffer).read_term()
}

trait TermReader {
    fn read_term(&mut self) -> Result<EncodedTerm, io::Error>;
}

impl<R: Read> TermReader for R {
    fn read_term(&mut self) -> Result<EncodedTerm, io::Error> {
        let mut type_buffer = [0];
        self.read_exact(&mut type_buffer)?;
        match type_buffer[0] {
            TYPE_NAMED_NODE_ID => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::NamedNode {
                    iri_id: StrHash::from_be_bytes(buffer),
                })
            }
            TYPE_NUMERICAL_BLANK_NODE_ID => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::NumericalBlankNode {
                    id: u128::from_be_bytes(buffer),
                })
            }
            TYPE_SMALL_BLANK_NODE_ID => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::SmallBlankNode(
                    SmallString::from_be_bytes(buffer).map_err(invalid_data_error)?,
                ))
            }
            TYPE_BIG_BLANK_NODE_ID => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::BigBlankNode {
                    id_id: StrHash::from_be_bytes(buffer),
                })
            }
            TYPE_SMALL_SMALL_LANG_STRING_LITERAL => {
                let mut language_buffer = [0; 16];
                self.read_exact(&mut language_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::SmallSmallLangStringLiteral {
                    value: SmallString::from_be_bytes(value_buffer).map_err(invalid_data_error)?,
                    language: SmallString::from_be_bytes(language_buffer)
                        .map_err(invalid_data_error)?,
                })
            }
            TYPE_SMALL_BIG_LANG_STRING_LITERAL => {
                let mut language_buffer = [0; 16];
                self.read_exact(&mut language_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::SmallBigLangStringLiteral {
                    value: SmallString::from_be_bytes(value_buffer).map_err(invalid_data_error)?,
                    language_id: StrHash::from_be_bytes(language_buffer),
                })
            }
            TYPE_BIG_SMALL_LANG_STRING_LITERAL => {
                let mut language_buffer = [0; 16];
                self.read_exact(&mut language_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::BigSmallLangStringLiteral {
                    value_id: StrHash::from_be_bytes(value_buffer),
                    language: SmallString::from_be_bytes(language_buffer)
                        .map_err(invalid_data_error)?,
                })
            }
            TYPE_BIG_BIG_LANG_STRING_LITERAL => {
                let mut language_buffer = [0; 16];
                self.read_exact(&mut language_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::BigBigLangStringLiteral {
                    value_id: StrHash::from_be_bytes(value_buffer),
                    language_id: StrHash::from_be_bytes(language_buffer),
                })
            }
            TYPE_SMALL_TYPED_LITERAL => {
                let mut datatype_buffer = [0; 16];
                self.read_exact(&mut datatype_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::SmallTypedLiteral {
                    datatype_id: StrHash::from_be_bytes(datatype_buffer),
                    value: SmallString::from_be_bytes(value_buffer).map_err(invalid_data_error)?,
                })
            }
            TYPE_BIG_TYPED_LITERAL => {
                let mut datatype_buffer = [0; 16];
                self.read_exact(&mut datatype_buffer)?;
                let mut value_buffer = [0; 16];
                self.read_exact(&mut value_buffer)?;
                Ok(EncodedTerm::BigTypedLiteral {
                    datatype_id: StrHash::from_be_bytes(datatype_buffer),
                    value_id: StrHash::from_be_bytes(value_buffer),
                })
            }
            TYPE_SMALL_STRING_LITERAL => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::SmallStringLiteral(
                    SmallString::from_be_bytes(buffer).map_err(invalid_data_error)?,
                ))
            }
            TYPE_BIG_STRING_LITERAL => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::BigStringLiteral {
                    value_id: StrHash::from_be_bytes(buffer),
                })
            }
            TYPE_BOOLEAN_LITERAL_TRUE => Ok(EncodedTerm::BooleanLiteral(true)),
            TYPE_BOOLEAN_LITERAL_FALSE => Ok(EncodedTerm::BooleanLiteral(false)),
            TYPE_FLOAT_LITERAL => {
                let mut buffer = [0; 4];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::FloatLiteral(f32::from_be_bytes(buffer)))
            }
            TYPE_DOUBLE_LITERAL => {
                let mut buffer = [0; 8];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DoubleLiteral(f64::from_be_bytes(buffer)))
            }
            TYPE_INTEGER_LITERAL => {
                let mut buffer = [0; 8];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::IntegerLiteral(i64::from_be_bytes(buffer)))
            }
            TYPE_DECIMAL_LITERAL => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DecimalLiteral(Decimal::from_be_bytes(buffer)))
            }
            TYPE_DATE_TIME_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DateTimeLiteral(DateTime::from_be_bytes(
                    buffer,
                )))
            }
            TYPE_TIME_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::TimeLiteral(Time::from_be_bytes(buffer)))
            }
            TYPE_DATE_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DateLiteral(Date::from_be_bytes(buffer)))
            }
            TYPE_G_YEAR_MONTH_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::GYearMonthLiteral(GYearMonth::from_be_bytes(
                    buffer,
                )))
            }
            TYPE_G_YEAR_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::GYearLiteral(GYear::from_be_bytes(buffer)))
            }
            TYPE_G_MONTH_DAY_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::GMonthDayLiteral(GMonthDay::from_be_bytes(
                    buffer,
                )))
            }
            TYPE_G_DAY_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::GDayLiteral(GDay::from_be_bytes(buffer)))
            }
            TYPE_G_MONTH_LITERAL => {
                let mut buffer = [0; 18];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::GMonthLiteral(GMonth::from_be_bytes(buffer)))
            }
            TYPE_DURATION_LITERAL => {
                let mut buffer = [0; 24];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DurationLiteral(Duration::from_be_bytes(
                    buffer,
                )))
            }
            TYPE_YEAR_MONTH_DURATION_LITERAL => {
                let mut buffer = [0; 8];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::YearMonthDurationLiteral(
                    YearMonthDuration::from_be_bytes(buffer),
                ))
            }
            TYPE_DAY_TIME_DURATION_LITERAL => {
                let mut buffer = [0; 16];
                self.read_exact(&mut buffer)?;
                Ok(EncodedTerm::DayTimeDurationLiteral(
                    DayTimeDuration::from_be_bytes(buffer),
                ))
            }
            _ => Err(invalid_data_error("the term buffer has an invalid type id")),
        }
    }
}
