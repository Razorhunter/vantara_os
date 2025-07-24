use std::env;

#[derive(Debug)]
pub struct Options {
    pub show_user: bool, // -u
    pub show_all: bool,           // ps -e / ps -ax
    pub only_with_tty: bool,      // ps -a
    pub only_without_tty: bool,   // ps -x
    pub custom_fields: Option<Vec<String>>, // -o pid,user,...
    pub show_version: bool,
    pub show_usage: bool,
    pub show_tree: bool, //Show as tree view
}

impl Options {
    pub fn parse() -> Self {
        let mut show_all = false;
        let mut show_user = false;
        let mut only_with_tty = false;
        let mut only_without_tty = false;
        let mut custom_fields = None;
        let mut show_version = false;
        let mut show_usage = false;
        let mut show_tree = false;

        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" => show_usage =  true,
                "--version" => show_version = true,
                _ if arg.starts_with('-') => {
                    for c in arg.chars().skip(1) {
                        match c {
                            'a' => only_with_tty = true,
                            'u' => show_user = true,
                            'x' => only_without_tty = true,
                            'e' => show_all = true,
                            'o' => {
                                if let Some(fields) = args.next() {
                                    custom_fields = Some(fields.split(',').map(|s| s.to_string()).collect());
                                }
                            },
                            't' => show_tree = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if only_with_tty && only_without_tty {
            only_with_tty = false;
            only_without_tty = false;
        }

        Options {
            show_all,
            only_with_tty,
            only_without_tty,
            show_user,
            custom_fields,
            show_version,
            show_usage,
            show_tree
        }
    }
}
