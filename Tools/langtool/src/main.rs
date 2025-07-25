use std::io;

mod section;
use section::{line_value, Section};

mod inifile;
use inifile::IniFile;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
    #[arg(short, long)]
    dry_run: bool,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser, Debug)]
enum Command {
    CopyMissingLines {
        #[arg(short, long)]
        dont_comment_missing: bool,
    },
    CommentUnknownLines {},
    RemoveUnknownLines {},
    AddNewKey {
        section: String,
        key: String,
    },
    AddNewKeyValue {
        section: String,
        key: String,
        value: String,
    },
    MoveKey {
        old: String,
        new: String,
        key: String,
    },
    CopyKey {
        old: String,
        new: String,
        key: String,
    },
    DupeKey {
        section: String,
        old: String,
        new: String,
    },
    RenameKey {
        section: String,
        old: String,
        new: String,
    },
    SortSection {
        section: String,
    },
    RemoveKey {
        section: String,
        key: String,
    },
    GetNewKeys,
    ImportSingle {
        filename: String,
        section: String,
        key: String,
    },
}

fn copy_missing_lines(
    reference_ini: &IniFile,
    target_ini: &mut IniFile,
    comment_missing: bool,
) -> io::Result<()> {
    for reference_section in &reference_ini.sections {
        // Insert any missing full sections.
        if !target_ini.insert_section_if_missing(reference_section) {
            if let Some(target_section) = target_ini.get_section_mut(&reference_section.name) {
                for line in &reference_section.lines {
                    target_section.insert_line_if_missing(line);
                }

                //target_section.remove_lines_if_not_in(reference_section);
                if comment_missing {
                    target_section.comment_out_lines_if_not_in(reference_section);
                }
            }
        } else {
            // Note: insert_section_if_missing will copy the entire section,
            // no need to loop over the lines here.
            println!("Inserted missing section: {}", reference_section.name);
        }
    }
    Ok(())
}

fn deal_with_unknown_lines(
    reference_ini: &IniFile,
    target_ini: &mut IniFile,
    remove: bool,
) -> io::Result<()> {
    for reference_section in &reference_ini.sections {
        if let Some(target_section) = target_ini.get_section_mut(&reference_section.name) {
            if remove {
                target_section.remove_lines_if_not_in(reference_section);
            } else {
                target_section.comment_out_lines_if_not_in(reference_section);
            }
        }
    }
    Ok(())
}

fn print_keys_if_not_in(
    reference_ini: &IniFile,
    target_ini: &mut IniFile,
    header: &str,
) -> io::Result<()> {
    for reference_section in &reference_ini.sections {
        if let Some(target_section) = target_ini.get_section_mut(&reference_section.name) {
            let keys = target_section.get_keys_if_not_in(reference_section);
            if !keys.is_empty() {
                println!("{} ({})", reference_section.name, header);
                for key in &keys {
                    println!("- {key}");
                }
            }
        }
    }
    Ok(())
}

fn move_key(target_ini: &mut IniFile, old: &str, new: &str, key: &str) -> io::Result<()> {
    if let Some(old_section) = target_ini.get_section_mut(old) {
        if let Some(line) = old_section.remove_line(key) {
            if let Some(new_section) = target_ini.get_section_mut(new) {
                new_section.insert_line_if_missing(&line);
            } else {
                println!("No new section {new}");
            }
        } else {
            println!("No key {key} in section {old}");
        }
    } else {
        println!("No old section {old}");
    }
    Ok(())
}

fn copy_key(target_ini: &mut IniFile, old: &str, new: &str, key: &str) -> io::Result<()> {
    if let Some(old_section) = target_ini.get_section_mut(old) {
        if let Some(line) = old_section.get_line(key) {
            if let Some(new_section) = target_ini.get_section_mut(new) {
                new_section.insert_line_if_missing(&line);
            } else {
                println!("No new section {new}");
            }
        } else {
            println!("No key {key} in section {old}");
        }
    } else {
        println!("No old section {old}");
    }
    Ok(())
}

fn remove_key(target_ini: &mut IniFile, section: &str, key: &str) -> io::Result<()> {
    if let Some(old_section) = target_ini.get_section_mut(section) {
        old_section.remove_line(key);
    } else {
        println!("No section {section}");
    }
    Ok(())
}

fn add_new_key(target_ini: &mut IniFile, section: &str, key: &str, value: &str) -> io::Result<()> {
    if let Some(section) = target_ini.get_section_mut(section) {
        section.insert_line_if_missing(&format!("{key} = {value}"));
    } else {
        println!("No section {section}");
    }
    Ok(())
}

fn rename_key(target_ini: &mut IniFile, section: &str, old: &str, new: &str) -> io::Result<()> {
    if let Some(section) = target_ini.get_section_mut(section) {
        section.rename_key(old, new);
    } else {
        println!("No section {section}");
    }
    Ok(())
}

fn dupe_key(target_ini: &mut IniFile, section: &str, old: &str, new: &str) -> io::Result<()> {
    if let Some(section) = target_ini.get_section_mut(section) {
        section.dupe_key(old, new);
    } else {
        println!("No section {section}");
    }
    Ok(())
}

fn sort_section(target_ini: &mut IniFile, section: &str) -> io::Result<()> {
    if let Some(section) = target_ini.get_section_mut(section) {
        section.sort();
    } else {
        println!("No section {section}");
    }
    Ok(())
}

// TODO: Look into using https://github.com/Byron/google-apis-rs/tree/main/gen/translate2 for initial translations.

fn main() {
    let opt = Args::parse();

    // TODO: Grab extra arguments from opt somehow.
    let args: Vec<String> = vec![]; //std::env::args().skip(1).collect();
    let mut filenames = args;

    let root = "../../assets/lang";
    let reference_ini_filename = "en_US.ini";

    let mut reference_ini =
        IniFile::parse(&format!("{root}/{reference_ini_filename}")).unwrap();

    if filenames.is_empty() {
        // Grab them all.
        for path in std::fs::read_dir(root).unwrap() {
            let path = path.unwrap();
            if path.file_name() == reference_ini_filename {
                continue;
            }
            let filename = path.file_name();
            let filename = filename.to_string_lossy();
            if !filename.ends_with(".ini") {
                continue;
            }
            filenames.push(path.file_name().to_string_lossy().to_string());
        }
    }

    let mut single_ini_section: Option<Section> = None;
    if let Command::ImportSingle {
        filename,
        section,
        key: _,
    } = &opt.cmd
    {
        if let Ok(single_ini) = IniFile::parse(filename) {
            if let Some(single_section) = single_ini.get_section("Single") {
                single_ini_section = Some(single_section.clone());
            } else {
                println!("No section {section} in {filename}");
            }
        } else {
            println!("Failed to parse {filename}");
            return;
        }
    }

    for filename in filenames {
        let reference_ini = &reference_ini;
        if filename == "langtool" {
            // Get this from cargo run for some reason.
            continue;
        }
        let target_ini_filename = format!("{root}/{filename}");
        if opt.verbose {
            println!("Langtool processing {target_ini_filename}");
        }

        let mut target_ini = IniFile::parse(&target_ini_filename).unwrap();

        match opt.cmd {
            Command::CopyMissingLines {
                dont_comment_missing,
            } => {
                copy_missing_lines(reference_ini, &mut target_ini, !dont_comment_missing).unwrap();
            }
            Command::CommentUnknownLines {} => {
                deal_with_unknown_lines(reference_ini, &mut target_ini, false).unwrap();
            }
            Command::RemoveUnknownLines {} => {
                deal_with_unknown_lines(reference_ini, &mut target_ini, true).unwrap();
            }
            Command::GetNewKeys => {
                print_keys_if_not_in(reference_ini, &mut target_ini, &target_ini_filename).unwrap();
            }
            Command::SortSection { ref section } => sort_section(&mut target_ini, section).unwrap(),
            Command::RenameKey {
                ref section,
                ref old,
                ref new,
            } => rename_key(&mut target_ini, section, old, new).unwrap(),
            Command::AddNewKey {
                ref section,
                ref key,
            } => add_new_key(&mut target_ini, section, key, key).unwrap(),
            Command::AddNewKeyValue {
                ref section,
                ref key,
                ref value,
            } => add_new_key(&mut target_ini, section, key, value).unwrap(),
            Command::MoveKey {
                ref old,
                ref new,
                ref key,
            } => {
                move_key(&mut target_ini, old, new, key).unwrap();
            }
            Command::CopyKey {
                // Copies between sections
                ref old,
                ref new,
                ref key,
            } => {
                copy_key(&mut target_ini, old, new, key).unwrap();
            }
            Command::DupeKey {
                ref section,
                ref old,
                ref new,
            } => {
                dupe_key(&mut target_ini, section, old, new).unwrap();
            }
            Command::RemoveKey {
                ref section,
                ref key,
            } => {
                remove_key(&mut target_ini, section, key).unwrap();
            }
            Command::ImportSingle {
                filename: _,
                ref section,
                ref key,
            } => {
                let lang_id = filename.strip_suffix(".ini").unwrap();
                if let Some(single_section) = &single_ini_section {
                    if let Some(target_section) = target_ini.get_section_mut(section) {
                        if let Some(single_line) = single_section.get_line(lang_id) {
                            if let Some(value) = line_value(&single_line) {
                                println!(
                                    "Inserting value {value} for key {key} in section {section} in {target_ini_filename}"
                                );
                                if !target_section
                                    .insert_line_if_missing(&format!("{key} = {value}"))
                                {
                                    // Didn't insert it, so it exists. We need to replace it.
                                    target_section.set_value(key, value);
                                }
                            }
                        } else {
                            println!("No lang_id {lang_id} in single section");
                        }
                    } else {
                        println!("No section {section} in {target_ini_filename}");
                    }
                } else {
                    println!("No section {section} in {filename}");
                }
            }
        }

        if !opt.dry_run {
            target_ini.write().unwrap();
        }
    }

    println!("Langtool processing {reference_ini_filename}");

    // Some commands also apply to the reference ini.
    match opt.cmd {
        Command::AddNewKey {
            ref section,
            ref key,
        } => {
            add_new_key(&mut reference_ini, section, key, key).unwrap();
        }
        Command::AddNewKeyValue {
            ref section,
            ref key,
            ref value,
        } => {
            add_new_key(&mut reference_ini, section, key, value).unwrap();
        }
        Command::SortSection { ref section } => sort_section(&mut reference_ini, section).unwrap(),
        Command::RenameKey {
            ref section,
            ref old,
            ref new,
        } => {
            if old == new {
                println!("WARNING: old == new");
            }
            rename_key(&mut reference_ini, section, old, new).unwrap();
        }
        Command::MoveKey {
            ref old,
            ref new,
            ref key,
        } => {
            move_key(&mut reference_ini, old, new, key).unwrap();
        }
        Command::CopyKey {
            // between sections
            ref old,
            ref new,
            ref key,
        } => {
            copy_key(&mut reference_ini, old, new, key).unwrap();
        }
        Command::DupeKey {
            // Inside a section, preserving a value
            ref section,
            ref old,
            ref new,
        } => {
            dupe_key(&mut reference_ini, section, old, new).unwrap();
        }
        Command::RemoveKey {
            ref section,
            ref key,
        } => {
            remove_key(&mut reference_ini, section, key).unwrap();
        }
        _ => {}
    }

    if !opt.dry_run {
        reference_ini.write().unwrap();
    }
}
