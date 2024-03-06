use std::convert::Infallible;
use std::io;
use std::ops::{ControlFlow, FromResidual, Try};
use std::ops::ControlFlow::{Break, Continue};
use std::process::Termination;

use thiserror::Error;

use crate::USAGE;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("too many arguments provided: {0}")]
    TooManyArguments(usize),
}

#[derive(Error, Debug)]
pub enum RispError {
    #[error("{0}\n{}", USAGE)]
    CliError(CliError),
    #[error("unable to open file {0}")]
    UnableToOpenFile(#[from] io::Error),

}

pub struct RispResult<T>(Result<T, RispError>);

impl RispResult<()> {
    pub fn _ok() -> RispResult<()> {
        RispResult { 0: Ok(()) }
    }
}

impl<T> Try for RispResult<T> {
    type Output = T;
    type Residual = RispError;

    fn from_output(output: Self::Output) -> Self {
        RispResult {
            0: Ok(output),
        }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(residual) => { Continue(residual) }
            Err(err) => { Break(err) }
        }
    }
}

impl<T> FromResidual<RispError> for RispResult<T> {
    fn from_residual(residual: RispError) -> Self {
        Self {
            0: Err(residual)
        }
    }
}

impl<T> FromResidual<Result<Infallible, RispError>> for RispResult<T> {
    fn from_residual(residual: Result<Infallible, RispError>) -> Self {
        Self {
            0: Err(residual.err().unwrap())
        }
    }
}

impl<T> FromResidual<Result<Infallible, std::io::Error>> for RispResult<T> {
    fn from_residual(residual: Result<Infallible, std::io::Error>) -> Self {
        Self {
            0: Err(residual.err().unwrap().into())
        }
    }
}


impl<T> FromResidual<Result<(), RispError>> for RispResult<T> {
    fn from_residual(residual: Result<(), RispError>) -> Self {
        Self {
            0: Err(residual.err().unwrap())
        }
    }
}

impl Termination for RispResult<()> {
    fn report(self) -> std::process::ExitCode {
        match self.0 {
            Ok(()) => { std::process::ExitCode::SUCCESS.report() }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::ExitCode::FAILURE.report()
            }
        }
    }
}