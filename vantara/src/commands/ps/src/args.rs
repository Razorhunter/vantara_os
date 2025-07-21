use std::env;

#[derive(Debug)]
pub struct Options {
    pub show_user: bool, // -u
    pub show_all: bool,           // ps -e / ps -ax
    pub only_with_tty: bool,      // ps -a
    pub only_without_tty: bool,   // ps -x
    pub custom_fields: Option<Vec<String>>, // -o pid,user,...
}

impl Options {
    pub fn parse() -> Self {
        let mut show_all = false;
        let mut show_user = false;
        let mut only_with_tty = false;
        let mut only_without_tty = false;
        let mut custom_fields = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg.starts_with('-') && arg.len() > 1 {
                for c in arg.chars().skip(1) {
                    match c {
                        'a' => only_with_tty = true,
                        'x' => only_without_tty = true,
                        'e' => show_all = true,
                        'u' => show_user = true,
                        'o' => {
                            if let Some(fields) = args.next() {
                                custom_fields = Some(fields.split(',').map(|s| s.to_string()).collect());
                            }
                        },
                        _ => {},
                    }
                }
            }
        }

        Options {
            show_all,
            only_with_tty,
            only_without_tty,
            show_user,
            custom_fields,
        }
    }
}
