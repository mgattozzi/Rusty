#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(unused_imports)] //for process::logic::*;

use std::process::*;
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::io::Result;
use process::logic::*;

///Interpret
///Given an input command, interpret parses and determines what and how
///to execute it and returns output or error output
pub fn interpret(command: Vec<&str>) -> String {

    //Refactoring
    //Break commands by logic
    //Run commands by logic precedence by looping through all of them here
    //output results
    //Create more functions for dealing with ands etc.
    let mut pipes = false;
    for i in command.clone() {
       if i.contains('|') {
           pipes = true;
           break;
        }
    }
    let output: Option<Output>;
    if pipes { //Pipe or no pipe
        output = piped(command);
    } else { //execute normally
        output = execute(command);
    }

    get_stdout_or_stderr(output)
}

///Execute
///Runs commands passed to it and returns the output
pub fn execute(command: Vec<&str>) -> Option<Output>{
    let args = command.as_slice();
    if args.len() > 1 {
        Command::new(&args[0]).args(&args[1.. ]).output().ok()
    } else if args.len() == 1{
        Command::new(&args[0]).output().ok()
    } else {
        Command::new("").output().ok()
    }
 }

///Get Stdout or Err
///Returns the standard output or error of an executed command or returns that
///the command was invalid
pub fn get_stdout_or_stderr(output: Option<Output>) -> String {
    match output.is_some(){
        true => {
            let temp = output.expect("Output has been checked");
            if temp.stdout.is_empty(){
                String::from_utf8(temp.stderr)
                    .expect("Should have translated to string easily")
            } else {
                String::from_utf8(temp.stdout)
                    .expect("Should have translated to string easily")
            }
        },
        false => "Please input a valid command".to_owned()
    }
}

pub fn get_status(output: Option<Output>) -> bool{
    match output.is_some(){
        true => {
            let temp = output.expect("Output has been checked");
            temp.status.success()
        },
        false => false,
    }
}

///Split Pipes
///Given a command with pipes in it, it will split them into separate
///commands to be executed
fn split_pipes(input: Vec<&str>) -> Vec<Vec<&str>> {
    let input_slice = input.as_slice();
    let mut pipe_commands: Vec<Vec<&str>> = Vec::new();
    let mut temp: Vec<&str> = Vec::new();
    for i in input_slice {
        if i.contains('|') {
            let mut splits = i.split('|');
            temp.push(splits.next()
                      .expect("Failed to split pipes"));
            if temp.last()
                .expect("Called last on an empty vec") == &""{
                temp.pop();
            }
            pipe_commands.push(temp.clone());
            temp.clear();
            temp.push(splits.next()
                      .expect("Unwrapped a non existent value"));
            if temp.last()
                .expect("Unwrapped an empty list") == &""{
                temp.pop();
            }
        } else {
            temp.push(i);
        }
    }
    pipe_commands.push(temp);
    pipe_commands
}

///Piped
///The logic of piping is done here and calls the functions that execute
///the pipes and returns the result
fn piped(input: Vec<&str>) -> Option<Output> {
    let mut split = split_pipes(input);
    let mut child_result = first_pipe(split.remove(0));
    let mut child: Child;

    //The assumption is that this will always execute properly
    //but to make sure nopipe_commands goes wrong an assert statement
    //has been added to make sure of this
    assert!(child_result.is_ok());
    child = child_result.ok().expect("Failed to unwrap an Result");

    while split.len() > 1 {
        child_result = execute_pipe(split.remove(0), child);
        assert!(child_result.is_ok());
        child = child_result.ok().expect("Failed to unwrap an Result");
    }

    final_pipe(split.remove(0), child)
}

///First Pipe
///Always executed if piping and returns the child process to be used
///for the next pipe.
fn first_pipe(command: Vec<&str>) -> Result<Child> {
    let args = command.as_slice();
    if args.len() > 1 {
        Command::new(&args[0]).args(&args[1.. ])
            .stdout(Stdio::piped()).spawn()
    } else if args.len() == 1{
        Command::new(&args[0])
            .stdout(Stdio::piped()).spawn()
    } else {
        Command::new("")
            .stdout(Stdio::piped()).spawn()
    }
}

///Execute Pipe
///Used if there are more than two commands with piping. Takes a Child process
///as input for the next pipe and returns a Child process.
fn execute_pipe(command: Vec<&str>, child: Child) -> Result<Child> {
    let args = command.as_slice();
    unsafe{
        let output = if args.len() > 1 {
            Command::new(&args[0]).args(&args[1.. ])
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout").as_raw_fd()))
                .spawn()
        } else if args.len() == 1{
            Command::new(&args[0])
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout").as_raw_fd()))
                .spawn()
        } else {
            Command::new("")
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout").as_raw_fd()))
                .spawn()
        };
        output
    }

}

///Final Pipe
///Always executed when piping processes. Takes a child process as input
///and returns the output of piping the commands.
fn final_pipe(command: Vec<&str>, child: Child) -> Option<Output> {
    let args = command.as_slice();
    unsafe{
        let output = if args.len() > 1 {
            Command::new(&args[0]).args(&args[1.. ])
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout for child process")
                        .as_raw_fd()))
                .output()
        } else if args.len() == 1{
            Command::new(&args[0])
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout for child process")
                        .as_raw_fd()))
                .output()
        } else {
            Command::new("")
                .stdout(Stdio::piped())
                .stdin(Stdio::from_raw_fd(child.stdout
                        .expect("No stdout for child process")
                        .as_raw_fd()))
                .output()
        };
        output.ok()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn pipes() {
        let vec: Vec<&str> = "ls /|grep bin| sort -r"
            .trim().split(' ').collect();
        let result = interpret(vec);
        assert_eq!("sbin\nbin\n",result);
     }

    #[test]
    #[should_panic]
    fn pipes_fail() {
        let vec: Vec<&str> = "ls |grep bin| sort -r"
            .trim().split(' ').collect();
        let result = interpret(vec);
        assert_eq!("Please input a valid command",result);
    }

    #[test]
    fn execute_test(){
        let vec: Vec<&str> = "ls -al"
            .trim().split(' ').collect();
        let result = interpret(vec);
        assert!(!result.is_empty());

    }

    #[test]
    fn execute_fail(){
        let vec: Vec<&str> = "blah"
            .trim().split(' ').collect();
        let result = interpret(vec);
        assert_eq!("Please input a valid command",result);
    }
}
