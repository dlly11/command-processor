// This module contains a command processor
#![cfg_attr(not(test), no_std)]
use heapless::{String, Vec};

use core::fmt::Write;

/// Return codes for commands
#[derive(Debug, PartialEq)]
pub enum ReturnCode {
    Success,
    Failure,
}

/// Return type for command callbacks
pub type CommandCallbackReturn<'a> = Result<ReturnCode, CommandProcessorError>;

/// Command callback type
pub type CommandCallback<'a> = fn(Option<&mut (dyn Write + 'a)>) -> CommandCallbackReturn<'a>;

/// A command item
///
/// # Arguments
///
/// * `HELP_STR_SIZE` - The maximum size of the help string
///
struct CommandItem<'a, const HELP_STR_SIZE: usize> {
    command: String<32>,
    callback: CommandCallback<'a>,
    help: Option<String<HELP_STR_SIZE>>,
}

/// A command processor
///
/// # Arguments
///
/// * `NUM_COMMANDS` - The maximum number of commands the processor can hold
/// * `HELP_STR_SIZE` - The maximum size of the help string
///
/// # Example
///
/// ```
/// use command_processor::{CommandProcessor, CommandProcessorError, ReturnCode, CommandCallbackReturn};
/// use heapless::String;
/// use core::fmt::Write;
///
/// fn printer_demo<'a>(
///    _: Option<&mut (dyn Write + 'a)>,
/// ) -> CommandCallbackReturn<'a> {
///    Ok(ReturnCode::Success)
/// }
///
/// let mut command_processor: CommandProcessor<8, 32> = CommandProcessor::new();
///
/// command_processor.add_command(
///     String::<32>::from("printer"),
///     printer_demo,
///     Some(String::<32>::from("Prints a message")),
/// ).unwrap();
///     
/// let mut writer: String<32> = String::new();
///
/// command_processor.process_command(&String::from("help"), Some(&mut writer)).unwrap();
///
/// assert_eq!(writer, "Prints a message\n");
///
/// writer.clear();
///
/// ```
///
pub struct CommandProcessor<'a, const NUM_COMMANDS: usize, const HELP_STR_SIZE: usize> {
    commands: Vec<CommandItem<'a, HELP_STR_SIZE>, NUM_COMMANDS>,
}

/// Errors that can occur when using the command processor
#[derive(Debug)]
pub enum CommandProcessorError {
    CommandAlreadyExists,
    CommandNotFound,
    CommandListFull,
    WriteError,
    NoWriter,
}

impl<'a, const NUM_COMMANDS: usize, const HELP_STR_SIZE: usize> Default
    for CommandProcessor<'a, NUM_COMMANDS, HELP_STR_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const NUM_COMMANDS: usize, const HELP_STR_SIZE: usize>
    CommandProcessor<'a, NUM_COMMANDS, HELP_STR_SIZE>
{
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Adds a command to the command processor
    ///
    /// # Arguments
    ///
    /// * `command` - The command to add
    /// * `callback` - The callback to call when the command is processed
    /// * `help` - The help string for the command
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the command was added successfully
    /// * `Err(CommandProcessorError::CommandAlreadyExists)` - If the command already exists
    /// * `Err(CommandProcessorError::CommandListFull)` - If the command list is full
    ///
    pub fn add_command(
        &mut self,
        command: String<32>,
        callback: CommandCallback<'a>,
        help: Option<String<HELP_STR_SIZE>>,
    ) -> Result<(), CommandProcessorError> {
        // Check if command already exists
        for cmd in self.commands.iter() {
            if cmd.command == command {
                return Err(CommandProcessorError::CommandAlreadyExists);
            }
        }

        self.commands
            .push(CommandItem {
                command,
                callback,
                help,
            })
            .map_err(|_| CommandProcessorError::CommandListFull)
    }

    /// Removes a command from the command processor
    ///
    /// # Arguments
    ///
    /// * `command` - The command to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the command was removed successfully
    /// * `Err(CommandProcessorError::CommandNotFound)` - If the command was not found
    ///
    pub fn remove_command(&mut self, command: String<32>) -> Result<(), CommandProcessorError> {
        for (i, cmd) in self.commands.iter().enumerate() {
            if cmd.command == command {
                self.commands.swap_remove(i);
                return Ok(());
            }
        }

        Err(CommandProcessorError::CommandNotFound)
    }

    /// Processes a command and calls the callback
    ///
    /// # Arguments
    ///
    /// * `command` - The command to process
    /// * `writer` - The writer the command can write with.
    ///
    /// # Returns
    ///
    /// * `Ok(ReturnCode)` - If the command was processed successfully
    /// * `Err(CommandProcessorError::CommandNotFound)` - If the command was not found
    /// * `Err(CommandProcessorError::NoWriter)` - If the command requires a writer but none was provided
    /// * `Err(CommandProcessorError::WriteError)` - If the command failed to write
    pub fn process_command(
        &mut self,
        command: &String<32>,
        writer: Option<&mut (dyn Write + 'a)>,
    ) -> Result<ReturnCode, CommandProcessorError> {
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

    fn help_printer(
        &mut self,
        writer: &mut (dyn Write + 'a),
    ) -> Result<ReturnCode, CommandProcessorError> {
        for cmd in self.commands.iter() {
            if let Some(help) = &cmd.help {
                writeln!(writer, "{}", help).map_err(|_| CommandProcessorError::WriteError)?;
            }
        }

        Ok(ReturnCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printer_demo<'a>(_: Option<&mut (dyn Write + 'a)>) -> CommandCallbackReturn<'a> {
        Ok(ReturnCode::Success)
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
        assert_eq!(result.unwrap(), ReturnCode::Success);
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
        assert_eq!(result.unwrap(), ReturnCode::Success);
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

        assert!(command_processor
            .process_command(&String::from("help"), Some(&mut buffer))
            .is_ok());

        assert_eq!(buffer, std::string::String::from("test: Test command\n"));
    }
}
