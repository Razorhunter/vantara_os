use std::fs::{File};
use std::io::{BufReader, stdout, Write, BufRead};
use std::process::exit;
use crossterm::{
    cursor::{Hide, MoveTo, Show, SetCursorStyle},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    terminal::size,
    ExecutableCommand,
};
use vantara::{safe_println, expand_wildcards, package_name, print_version, safe_eprintln};

enum Mode {
    Normal,
    Insert,
}

struct Options {
    number: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut options = Options{
        number: false
    };

    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "--number" => options.number = true,
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'n' => options.number = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !arg.starts_with('-') => paths.push(arg.clone()),
            _ => {
                safe_println(format_args!("{}: unknown option '{}'", package_name!(), arg));
                print_usage();
                exit(1);
            }
        }
    }

    if paths.is_empty() {
        safe_println(format_args!("{}: please specify at least one (1) file", package_name!()));
        print_usage();
        exit(1);
    }

    paths = expand_wildcards(&paths);

    let mut handle = stdout.lock();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(SetCursorStyle::BlinkingBlock)?;

    for filename in &paths {
        let mut lines: Vec<String> = if let Ok(file) = File::open(filename) {
            let lines: Vec<String> = BufReader::new(file).lines().filter_map(Result::ok).collect();
            if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            }
        } else {
            vec![String::new()]
        };

        enable_raw_mode()?;

        let mut cursor_x: usize = 0;
        let mut cursor_y: usize = 0;
        let mut scroll_offset = 0;
        let mut mode = Mode::Normal;

        let mut last_lines: Vec<String> = vec!["".to_string(); size()?.1 as usize];
        let mut last_screen_height = 0usize;
        let mut is_dirty: bool = false;
        let mut search_keyword: String = String::new();

        loop {
            let (term_width, term_height) = size()?;
            let screen_height = if term_height == 0 { 100 } else { term_height.saturating_sub(1) } as usize;
            let terminal_width = if term_width == 0 { 238 } else { term_width } as usize;

            execute!(handle, Hide)?;

            // Resize last_lines if terminal size changed
            if screen_height != last_screen_height {
                last_screen_height = screen_height;
                last_lines = vec!["".to_string(); screen_height];
                // Optional: force full redraw
                execute!(handle, Clear(ClearType::All))?;
            }

            let status_left = format!(
                "{} | ^S: Save | ^Q: Exit | ^W: Search | ^L: Go to Line",
                match mode {
                    Mode::Normal => "Insert: Edit",
                    Mode::Insert => "Esc: Exit edit",
                },
            );

            let status_right = format!(
                "File: {}{} | Row: {} Col: {} | W: {} H: {} | Mode: {}",
                filename,
                if is_dirty { '*' } else { ' ' },
                cursor_y + 1,
                cursor_x + 1,
                term_width,
                term_height,
                match mode {
                    Mode::Normal => "Read Only",
                    Mode::Insert => "Edit",
                },
            );

            let wrapped_lines = wrap_all_lines(&lines, terminal_width);
            for i in 0..screen_height {
                let visual_idx = scroll_offset + i;
                let content = if let Some((lineno, subline)) = wrapped_lines.get(visual_idx) {
                    if options.number {
                        format!("{:>3}: {}", lineno + 1, subline)
                    } else {
                        subline.clone()
                    }
                } else {
                    "~".to_string()
                };

                if content != last_lines[i] {
                    execute!(
                        handle,
                        MoveTo(0, i as u16),
                        Clear(ClearType::CurrentLine)
                    )?;
                    write!(handle, "{}", content)?;
                    last_lines[i] = content;
                }
            }

            let total_lines = wrapped_lines.len();
            let start_line = scroll_offset;
            let end_line = scroll_offset + screen_height;

            let show_up = start_line > 0;
            let show_down = end_line < total_lines;

            let scroll_indicator = match (show_up, show_down) {
                (true, true) => "↑↓",
                (true, false) => "↑ ",
                (false, true) => " ↓",
                (false, false) => "  ",
            };

            let spacing = terminal_width
                .saturating_sub(status_left.len() + status_right.len() + scroll_indicator.len() + 2);
            let padding = " ".repeat(spacing);

            execute!(handle, MoveTo(0, screen_height as u16))?;
            write!(handle, "{} {} {}{}", status_left, padding, status_right, scroll_indicator)?;

            // Show cursor after drawing everything
            let mut visual_cursor_y = 0;
            let mut visual_cursor_x = 0;

            for (i, (line_idx, subline)) in wrapped_lines.iter().enumerate() {
                if *line_idx == cursor_y {
                    let offset = cursor_x.min(lines[cursor_y].len());
                    let chunks = wrap_line(&lines[cursor_y], terminal_width);
                    let mut total = 0;
                    for (j, chunk) in chunks.iter().enumerate() {
                        if offset <= total + chunk.len() {
                            visual_cursor_y = i + j;
                            visual_cursor_x = offset - total;
                            break;
                        }
                        total += chunk.len();
                    }
                    break;
                }
            }

            let visual_x = (visual_cursor_x + if options.number { 5 } else { 0 }) as u16;
            let visual_y = (visual_cursor_y - scroll_offset) as u16;
            execute!(handle, MoveTo(visual_x, visual_y), Show)?;

            //Input command
            match read()? {
                //Quit editor
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    //Check if content being edited or not
                    if is_dirty {
                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                        write!(handle, "Save changes to '{}'? (Y/N): ", filename)?;
                        handle.flush()?;

                        loop {
                            if let Event::Key(KeyEvent {
                                code: KeyCode::Char(c),
                                ..
                            }) = read()?
                            {
                                match c.to_ascii_lowercase() {
                                    'y' => {
                                        if let Ok(mut file) = File::create(filename) {
                                            for line in &lines {
                                                writeln!(file, "{}", line)?;
                                            }
                                        }
                                        break;
                                    }
                                    'n' => {
                                        break;
                                    }
                                    _ => {} //Ignore
                                }
                            }
                        }

                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    }

                    disable_raw_mode()?;
                    stdout.execute(Show)?;
                    stdout.execute(Clear(ClearType::All))?;
                    stdout.execute(MoveTo(0, 0))?;
                    break;
                }
                //Save changes
                Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) if matches!(mode, Mode::Insert) => {
                    execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    write!(handle, "Save changes to '{}'? (Y/N): ", filename)?;
                    handle.flush()?;

                    loop {
                        if let Event::Key(KeyEvent {
                            code: KeyCode::Char(c),
                            ..
                        }) = read()?
                        {
                            match c.to_ascii_lowercase() {
                                'y' => {
                                    if let Ok(mut file) = File::create(filename) {
                                        for line in &lines {
                                            writeln!(file, "{}", line)?;
                                        }
                                    }

                                    is_dirty = false;
                                    break;
                                }
                                'n' => {
                                    break;
                                }
                                _ => {} //Ignore
                            }
                        }
                    }
                    execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                }
                //Go to line
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    let mut input = String::new();
                    let mut canceled = false;

                    execute!(handle, MoveTo(0, screen_height as u16 + 1),Clear(ClearType::CurrentLine),)?;
                    write!(handle, "Enter line number (Esc to exit): ")?;
                    handle.flush()?;

                    loop {
                        if let Event::Key(key) = read()? {
                            match key.code {
                                KeyCode::Enter => break,
                                KeyCode::Backspace => {
                                    input.pop();
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    input.push(c);
                                }
                                KeyCode::Esc => {
                                    canceled = true;
                                    break;
                                }
                                _ => {} //Ignore
                            }

                            execute!(handle, MoveTo(33, screen_height as u16 + 1), Clear(ClearType::UntilNewLine),)?;
                            write!(handle, "{}", input)?;
                            handle.flush()?;
                        }
                    }

                    if canceled {
                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine),)?;
                    } else if let Ok(line_number) = input.parse::<usize>() {
                        if line_number > 0 && line_number <= lines.len() {
                            cursor_y = line_number - 1;
                            cursor_x = cursor_x.min(lines[cursor_y].len());

                            if cursor_y >= scroll_offset + screen_height {
                                scroll_offset = cursor_y.saturating_sub(screen_height - 1);
                            } else if cursor_y < scroll_offset {
                                scroll_offset = cursor_y;
                            }
                        }
                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    }
                }
                //Search keyword
                Event::Key(KeyEvent {
                    code: KeyCode::Char('w'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    let mut input = String::new();
                    let mut canceled = false;
                    let mut pos = 0; //To set where cursor will be printed

                    execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    let mut prompt_search = format!("Enter keyword (Esc to exit): ");

                    if !search_keyword.is_empty() {
                        prompt_search = format!("Enter keyword (Esc to exit) [{}]: ", search_keyword);
                    }

                    pos = prompt_search.len() as u16;
                    write!(handle, "{}", prompt_search)?;

                    handle.flush()?;

                    loop {
                        if let Event::Key(key) = read()? {
                            match key.code {
                                KeyCode::Enter => break,
                                KeyCode::Backspace => {
                                    input.pop();
                                }
                                KeyCode::Char(c) => {
                                    if !c.is_control() {
                                        input.push(c);
                                    }
                                }
                                KeyCode::Esc => {
                                    canceled = true;
                                    break;
                                }
                                _ => {}
                            }

                            execute!(handle, MoveTo(pos, screen_height as u16 + 1), Clear(ClearType::UntilNewLine))?;
                            write!(handle, "{}", input)?;
                            handle.flush()?;
                        }
                    }

                    if canceled {
                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    } else {
                        if input.is_empty() && !search_keyword.is_empty() {
                            input = search_keyword.clone();
                        }

                        if !input.is_empty() {
                            let mut found = false;
                            search_keyword = input.clone(); //Stored previous search for future search

                            for (i, line) in lines.iter().enumerate().skip((if cursor_y == (lines.len() - 1) { 0 } else { cursor_y }) + 1) {
                                if let Some(pos) = line.find(&input) {
                                    cursor_y = i;
                                    cursor_x = pos;
                                    if cursor_y >= scroll_offset + screen_height {
                                        scroll_offset = cursor_y.saturating_sub(screen_height - 1);
                                    } else if cursor_y < scroll_offset {
                                        scroll_offset = cursor_y;
                                    }
                                    found = true;
                                    break;
                                }
                            }
                        }

                        execute!(handle, MoveTo(0, screen_height as u16 + 1), Clear(ClearType::CurrentLine))?;
                    }
                }
                // Enter edit mode
                Event::Key(KeyEvent {
                    code: KeyCode::Insert,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) if matches!(mode, Mode::Normal) => {
                    mode = Mode::Insert;
                }
                // Exit edit mode
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    mode = Mode::Normal;
                }
                // Navigate cursor x,y direction
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                }) => {
                    if cursor_y > 0 {
                        cursor_y -= 1;
                        cursor_x = cursor_x.min(lines[cursor_y].len());
                        if cursor_y < scroll_offset {
                            scroll_offset -= 1;
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => {
                    if cursor_y + 1 < lines.len() {
                        cursor_y += 1;
                        cursor_x = cursor_x.min(lines[cursor_y].len());
                        if cursor_y >= scroll_offset + screen_height {
                            scroll_offset += 1;
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => {
                    if cursor_x > 0 {
                        cursor_x -= 1;
                    } else {
                        if cursor_y > 0 {
                            cursor_y -= 1;
                        } else {
                            cursor_y = 0;
                        }
                        cursor_x = if cursor_y == 0 { 0 } else { lines[cursor_y].len() };
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => {
                    if cursor_x < lines[cursor_y].len() {
                        cursor_x += 1;
                    } else {
                        if cursor_y + 1 < lines.len() {
                            cursor_y += 1;
                            cursor_x = 0;
                        }
                    }
                }
                //Navigate cursor to beginning of line
                Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    ..
                }) => {
                    cursor_x = 0;
                }
                //Navigate cursor to end of line
                Event::Key(KeyEvent {
                    code: KeyCode::End,
                    ..
                }) => {
                    cursor_x = lines[cursor_y].len();
                }
                // Fungsi mod insert
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) if matches!(mode, Mode::Insert) => {
                    lines[cursor_y].insert(cursor_x, c);
                    cursor_x += 1;
                    is_dirty = true;
                }
                //Edit file
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) if matches!(mode, Mode::Insert) => {
                    let new_line = lines[cursor_y].split_off(cursor_x);
                    lines.insert(cursor_y + 1, new_line);
                    cursor_y += 1;
                    cursor_x = 0;
                    if cursor_y >= scroll_offset + screen_height {
                        scroll_offset += 1;
                    }
                    is_dirty = true;
                }
                //Backspace
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) if matches!(mode, Mode::Insert) => {
                    if cursor_x > 0 {
                        lines[cursor_y].remove(cursor_x - 1);
                        cursor_x -= 1;
                    } else if cursor_y > 0 {
                        let removed = lines.remove(cursor_y);
                        cursor_y -= 1;
                        cursor_x = lines[cursor_y].len();
                        lines[cursor_y].push_str(&removed);
                    }
                    is_dirty = true;
                }
                //Delete
                Event::Key(KeyEvent {
                    code: KeyCode::Delete,
                    ..
                }) if matches!(mode, Mode::Insert) => {
                    if cursor_x < lines[cursor_y].len() {
                        lines[cursor_y].remove(cursor_x);
                    } else if cursor_y + 1 < lines.len() {
                        let removed = lines.remove(cursor_y + 1);
                        lines[cursor_y].push_str(&removed);
                    }
                    is_dirty = true;
                }
                // PageDown
                Event::Key(KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                }) => {
                    let jump = screen_height;
                    cursor_y = (cursor_y + jump).min(lines.len().saturating_sub(1));
                    scroll_offset = cursor_y.saturating_sub(screen_height.saturating_sub(1));
                    cursor_x = cursor_x.min(lines.get(cursor_y).map(|l| l.len()).unwrap_or(0));
                }
                // PageUp
                Event::Key(KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                }) => {
                    let jump = screen_height;
                    cursor_y = cursor_y.saturating_sub(jump);
                    scroll_offset = cursor_y;
                    cursor_x = cursor_x.min(lines.get(cursor_y).map(|l| l.len()).unwrap_or(0));
                }
                _ => {}
            }
        }
    }

    stdout.execute(LeaveAlternateScreen)?;
    stdout.flush()?;

    Ok(())
}

fn wrap_line(line: &str, width: usize) -> Vec<String> {
    if width == 0 || line.is_empty() {
        return vec![line.to_string()];
    }

    line.chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn wrap_all_lines(lines: &[String], width: usize) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    for (i, line)  in lines.iter().enumerate() {
        let wrapped = wrap_line(line, width);
        for sub in wrapped {
            result.push((i, sub));
        }
    }
    result
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [FILES..]", package_name!()));
    safe_println(format_args!("     n | --number    Show number all lines"));
    safe_println(format_args!("     --help          Show help"));
    safe_println(format_args!("     --version       Show version"));
}
