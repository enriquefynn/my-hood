use std::fmt;

#[derive(Debug)]
pub struct HoodError {
    pub msg: String,
}

impl fmt::Display for HoodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// impl ResponseError for HoodError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::BadRequest().body(self.msg.clone())
//     }
// }
