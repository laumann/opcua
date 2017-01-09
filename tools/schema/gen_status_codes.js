var _ = require("lodash");
var csv = require("csv-parser");
var fs = require("fs");

var settings = require("./settings");

var status_code_csv = `${settings.schema_dir}/Opc.Ua.StatusCodes.csv`;

var rs_out = fs.createWriteStream(`${settings.rs_dir}/types/status_codes.rs`);

function write_to_rs(out, message) {
    out.write(message);
}

write_to_rs(rs_out,
    `// This file was autogenerated from Opc.Ua.StatusCodes.csv
use std;
use std::io::{Read, Write, Result};

use super::encodable_types::*;
use super::helpers::*;

#[derive(PartialEq, Debug, Clone)]
pub struct StatusCode {
    pub name: &'static str,
    pub code: u32,
    pub description: &'static str,
}

impl Encoder<StatusCode> for StatusCode {
    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u32(stream, self.code)
    }

    fn decode(stream: &mut Read) -> Result<StatusCode> {
        let code = read_u32(stream)?;
        let status_code = StatusCode::from_u32(code);
        if status_code.is_ok() { Ok(status_code.unwrap().clone()) } else { Ok(BAD_UNEXPECTED_ERROR.clone()) }
    }
}

impl StatusCode {
    /// Tests if the status code is bad
    pub fn is_bad(&self) -> bool {
        (self.code & 0x80000000) != 0
    }

    /// Tests if the status code is uncertain
    pub fn is_uncertain(&self) -> bool {
        (self.code & 0x40000000) != 0
    }

    /// Tests if the status code is good (i.e. not bad or uncertain)
    pub fn is_good(&self) -> bool {
        !self.is_bad() && !self.is_uncertain()
    }
`);

var status_codes = [
    {
        var_name: "GOOD",
        str_code: "Good",
        hex_code: 0,
        description: "Good"
    }
];

var csv_data = fs.createReadStream(status_code_csv)
    .pipe(csv(['str_code', 'hex_code', 'description']))
    .on('data', function (data) {
        data.var_name = _.toUpper(_.snakeCase(data.str_code));
        status_codes.push(data);
    })
    .on('end', function () {
        // Sort status
        status_codes = _.sortBy(status_codes, ['hex_code'])


      write_to_rs(rs_out,
`
    /// Takes an OPC UA status code as a UInt32 and returns the matching StatusCode, assuming there is one
    pub fn from_u32(code: u32) -> std::result::Result<&'static StatusCode, ()> {
        match code {
`);
      _.each(status_codes, function (data) {
        write_to_rs(rs_out, `            ${data.hex_code} => Ok(&${data.var_name}),\n`);
      });
      write_to_rs(rs_out,
`            _ => Err(())
        }
    }

    /// Takes an OPC UA status code as a string and returns the matching StatusCode - assuming there is one
    pub fn from_str(name: &str) -> std::result::Result<&'static StatusCode, ()> {
        match name {
`);
      _.each(status_codes, function (data) {
        write_to_rs(rs_out, `            "${data.str_code}" => Ok(&${data.var_name}),\n`);
      });
      write_to_rs(rs_out,
`            _ => Err(())
        }
    }
}

`);

      _.each(status_codes, function (data) {
            write_to_rs(rs_out,
`/// ${data.description}
pub static ${data.var_name}: StatusCode = StatusCode {
    name: "${data.str_code}",
    code: ${data.hex_code},
    description: "${data.description}"
};\n`);
        });

        write_to_rs(rs_out, ``);

    });