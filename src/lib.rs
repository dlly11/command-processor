// This module contains a command processor
#![cfg_attr(not(test), no_std)]
use heapless::{String, Vec};

use core::fmt::Write;

pub struct CommandItem<'a, const RESULT_STR_SIZE: usize> {
    command: String<32>,
    callback: fn(
        Option<&mut (dyn Write + 'a)>,
    ) -> Result<Option<String<RESULT_STR_SIZE>>, CommandProcessorError>,
    help: Option<String<RESULT_STR_SIZE>>,
}

pub struct CommandProcessor<'a, const NUM_COMMANDS: usize, const RESULT_STR_SIZE: usize> {
    commands: Vec<CommandItem<'a, RESULT_STR_SIZE>, NUM_COMMANDS>,
}

#[derive(Debug)]
pub enum CommandProcessorError {
    CommandAlreadyExists,
    CommandNotFound,
    CommandListFull,
    WriterError,
    NoWriter,
}

impl<'a, const NUM_COMMANDS: usize, const RESULT_STR_SIZE: usize>
    CommandProcessor<'a, NUM_COMMANDS, RESULT_STR_SIZE>
{
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_command(
        &mut self,
        command: String<32>,
        callback: fn(Option<&mut (dyn Write + 'a)>) -> Result<Option<String<RESULT_STR_SIZE>>, CommandProcessorError>,
        help: Option<String<RESULT_STR_SIZE>>,
    ) -> Result<(), CommandProcessorError> {
        // Check if command already exists
        for cmd in self.commands.iter() {
            if cmd.command == command {
                return Err(CommandProcessorError::CommandAlreadyExists);
            }
        }

        match self.commands.push(CommandItem {
            command,
            callback,
            help,
        }) {
            Ok(_) => Ok(()),
            Err(_) => Err(CommandProcessorError::CommandListFull),
        }
    }

    pub fn remove_command(&mut self, command: String<32>) -> Result<(), CommandProcessorError> {
        // Check if command already exists
        for (i, cmd) in self.commands.iter().enumerate() {
            if cmd.command == command {
                self.commands.swap_remove(i);
                return Ok(());
            }
        }

        Err(CommandProcessorError::CommandNotFound)
    }

    fn help_printer(
        &mut self,
        writer: &mut (dyn Write + 'a),
    ) -> Result<Option<String<RESULT_STR_SIZE>>, CommandProcessorError> {
        for cmd in self.commands.iter() {
            if let Some(help) = &cmd.help {
                match writeln!(writer, "{}", help) {
                    Ok(_) => (),
                    Err(_) => return Err(CommandProcessorError::WriterError),
                }
            }
        }

        Ok(None)
    }

    pub fn process_command(
        &mut self,
        command: &String<32>,
        writer: Option<&mut (dyn Write + 'a)>,
    ) -> Result<Option<String<RESULT_STR_SIZE>>, CommandProcessorError> {
        if command == "help" {
            match writer {
                Some(writer) => return self.help_printer(writer),
                None => return Err(CommandProcessorError::NoWriter),
            }
        }

        match self.commands.iter().find(|cmd| cmd.command == *command) {
            Some(cmd) => (cmd.callback)(writer),
            None => Err(CommandProcessorError::CommandNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printer_demo<'a>(_: Option<&mut (dyn Write + 'a)>) -> Result<Option<String<32>>, CommandProcessorError> {
        Ok(Some(String::from("Hello")))
    }

    #[test]
    fn test_command_processor() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        let result = command_processor.process_command(&String::from("test"), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(String::from("Hello")));
    }

    #[test]
    fn test_no_commands() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        let result = command_processor.process_command(&String::from("test"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_commands() {
        let mut command_processor: CommandProcessor<1, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        assert!(command_processor
            .add_command(
                String::from("test2"),
                printer_demo,
                Some(String::from("Test command 2"))
            )
            .is_err());
    }

    #[test]
    fn test_remove_command() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        assert!(command_processor
            .remove_command(String::from("test"))
            .is_ok());

        let result = command_processor.process_command(&String::from("test"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_command_not_found() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        assert!(command_processor
            .remove_command(String::from("test2"))
            .is_err());
    }

    #[test]
    fn test_writable_command() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        let mut buffer = std::string::String::new();
        let result = command_processor.process_command(&String::from("test"), Some(&mut buffer));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(String::from("Hello")));
    }

    #[test]
    fn test_unknown_command() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("Test command"))
            )
            .is_ok());

        let result = command_processor.process_command(&String::from("unknown"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_command() {
        let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();

        assert!(command_processor
            .add_command(
                String::from("test"),
                printer_demo,
                Some(String::from("test: Test command"))
            )
            .is_ok());

        let mut buffer = std::string::String::new();

        assert!(command_processor.process_command(&String::from("help"), Some(&mut buffer)).is_ok());

        assert_eq!(
            buffer,
            std::string::String::from("test: Test command\n")
        );
    }
}
