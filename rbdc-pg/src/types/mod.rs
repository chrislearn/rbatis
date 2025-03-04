/// pg types see https://www.postgresql.org/docs/current/datatype.html
pub mod oid;
use crate::type_info::PgTypeInfo;
pub use oid::Oid;
use rbs::Value;

pub mod array;
pub mod bigdecimal;
pub mod bool;
pub mod byte;
pub mod date;
pub mod datetime;
pub mod decimal;
pub mod decode;
pub mod encode;
pub mod float;
pub mod int;
pub mod json;
pub mod money;
pub mod numeric;
pub mod string;
pub mod time;
pub mod timestamp;
pub mod timestamptz;
pub mod timetz;
pub mod uuid;
pub mod value;

pub trait TypeInfo {
    fn type_info(&self) -> PgTypeInfo;
}

impl TypeInfo for Value {
    fn type_info(&self) -> PgTypeInfo {
        match self {
            Value::Null => PgTypeInfo::UNKNOWN,
            Value::Bool(_) => PgTypeInfo::BOOL,
            Value::I32(_) => PgTypeInfo::INT4,
            Value::I64(_) => PgTypeInfo::INT8,
            Value::U32(_) => PgTypeInfo::INT4,
            Value::U64(_) => PgTypeInfo::INT8,
            Value::F32(_) => PgTypeInfo::FLOAT4,
            Value::F64(_) => PgTypeInfo::FLOAT8,
            Value::String(_) => PgTypeInfo::VARCHAR,
            Value::Binary(_) => PgTypeInfo::BYTEA_ARRAY,
            Value::Array(arr) => {
                if arr.len() == 0 {
                    return PgTypeInfo::UNKNOWN;
                }
                arr[0]
                    .type_info()
                    .clone()
                    .to_array_element()
                    .unwrap_or(PgTypeInfo::UNKNOWN)
            }
            Value::Map(_) => PgTypeInfo::UNKNOWN,
            Value::Ext(type_name, _) => {
                match *type_name {
                    "Uuid" => PgTypeInfo::UUID,
                    //decimal = 12345678
                    "Decimal" => PgTypeInfo::NUMERIC,
                    //Date = "1993-02-06"
                    "Date" => PgTypeInfo::DATE,
                    //RFC3339NanoTime = "15:04:05.999999999"
                    "Time" => PgTypeInfo::TIME,
                    //RFC3339 = "2006-01-02 15:04:05.999999"
                    "Timestamp" => PgTypeInfo::TIMESTAMP,
                    "DateTime" => PgTypeInfo::TIMESTAMP,
                    "Bool" => PgTypeInfo::BOOL,
                    "Bytea" => PgTypeInfo::BYTEA,
                    "Char" => PgTypeInfo::CHAR,
                    "Name" => PgTypeInfo::NAME,
                    "Int8" => PgTypeInfo::INT8,
                    "Int2" => PgTypeInfo::INT2,
                    "Int4" => PgTypeInfo::INT4,
                    "Text" => PgTypeInfo::TEXT,
                    "Oid" => PgTypeInfo::OID,
                    "Json" => PgTypeInfo::JSON,
                    "Point" => PgTypeInfo::POINT,
                    "Lseg" => PgTypeInfo::LSEG,
                    "Path" => PgTypeInfo::PATH,
                    "Box" => PgTypeInfo::BOX,
                    "Polygon" => PgTypeInfo::POLYGON,
                    "Line" => PgTypeInfo::LINE,
                    "Cidr" => PgTypeInfo::CIDR,
                    "Float4" => PgTypeInfo::FLOAT4,
                    "Float8" => PgTypeInfo::FLOAT8,
                    "Unknown" => PgTypeInfo::UNKNOWN,
                    "Circle" => PgTypeInfo::CIRCLE,
                    "Macaddr8" => PgTypeInfo::MACADDR8,
                    "Macaddr" => PgTypeInfo::MACADDR,
                    "Inet" => PgTypeInfo::INET,
                    "Bpchar" => PgTypeInfo::BPCHAR,
                    "Varchar" => PgTypeInfo::VARCHAR,
                    "Timestamptz" => PgTypeInfo::TIMESTAMPTZ,
                    "Interval" => PgTypeInfo::INTERVAL,
                    "Timetz" => PgTypeInfo::TIMETZ,
                    "Bit" => PgTypeInfo::BIT,
                    "Varbit" => PgTypeInfo::VARBIT,
                    "Numeric" => PgTypeInfo::NUMERIC,
                    "Record" => PgTypeInfo::RECORD,
                    "Jsonb" => PgTypeInfo::JSONB,
                    "Int4Range" => PgTypeInfo::INT4_RANGE,
                    "NumRange" => PgTypeInfo::NUM_RANGE,
                    "TsRange" => PgTypeInfo::TS_RANGE,
                    "TstzRange" => PgTypeInfo::TSTZ_RANGE,
                    "DateRange" => PgTypeInfo::DATE_RANGE,
                    "Int8Range" => PgTypeInfo::INT8_RANGE,
                    "Jsonpath" => PgTypeInfo::JSONPATH,
                    "Money" => PgTypeInfo::MONEY,
                    "Void" => PgTypeInfo::VOID,
                    "Custom" => PgTypeInfo::UNKNOWN,
                    "DeclareWithName" => PgTypeInfo::UNKNOWN,
                    "DeclareWithOid" => PgTypeInfo::UNKNOWN,
                    _ => PgTypeInfo::UNKNOWN,
                }
            }
        }
    }
}
