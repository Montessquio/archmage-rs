use std::{future::Future, pin::Pin};

use eyre::{bail, Result};
use hashbrown::HashMap;
use serenity::{
    all::{Context, CreateCommand},
    model::prelude::*,
};
use tracing::info;

use crate::archmage::Archmage;

// mod music;
pub mod ping;
pub mod roll;
//pub mod pbp;

pub struct CommandDispatcher {
    commands: HashMap<String, (CreateCommand, HandleFn)>,
}

impl CommandDispatcher {
    /// Create a command dispatcher with no commands in it.
    /// Register new ones using [CommandDispatcher::register]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Add a new command to the dispatcher. Duplicate commands are not allowed
    /// and will return an `Error`.
    pub fn register<T>(&mut self) -> Result<()>
    where
        T: ArchmageCommand,
    {
        for (name, create_command, runner) in T::register() {
            info!("Registered {name}");
            if self.commands.contains_key(&name) {
                bail!("Dispatcher already contains command named {name}");
            }
            let _ = self.commands.insert(name, (create_command, runner));
        }

        Ok(())
    }

    /// Get the runnable function associated with the given key, or None.
    pub fn get_runner(&self, k: impl AsRef<str>) -> Option<&HandleFn> {
        self.commands.get(k.as_ref()).map(|(_, f)| f)
    }

    /// Shorthand to quickly run a given command. If the command is not present, returns None.
    pub async fn run(
        &self,
        k: impl AsRef<str>,
        server: &Archmage,
        command: &CommandInteraction,
        ctx: &Context,
    ) -> Option<Result<()>> {
        match self.get_runner(k) {
            None => None,
            Some(f) => Some(f(server, command, ctx).await),
        }
    }

    /// Get all metadata objects for all registered commands. Plural form of
    /// [CommandDispatcher::get_def].
    pub fn get_all_defs(&self) -> impl Iterator<Item = &CreateCommand> {
        self.commands.values().map(|(cc, _)| cc)
    }
}

pub type HandleFnResult = Result<(), eyre::Report>;
pub type HandleFnReturn<'a> = Pin<Box<dyn Future<Output=HandleFnResult> + Send + 'a>>;
pub type HandleFn = Box<dyn for<'a> Fn(&'a Archmage, &'a CommandInteraction, &'a Context) -> HandleFnReturn<'a> + Send + Sync>;
pub trait ArchmageCommand {
    fn register() -> Vec<(String, CreateCommand, HandleFn)>;
}

macro_rules! handle_fn {
    ($i:path) => {
        Box::new(|a, i, c| Box::pin($i(a, i, c)))
    }
}
pub(crate) use handle_fn;

macro_rules! registerable_tuples {
    ($($i:ident),+) => {
        #[allow(unused_parens)]
        impl<$( $i ),+> ArchmageCommand for ( $( $i ),+ , ) where $( $i: ArchmageCommand ),+ {
            fn register() -> Vec<(String, CreateCommand, HandleFn)> {
                vec![
                    $( $i::register() ),+
                ]
                .into_iter()
                .flatten()
                .collect()
            }
        }
    }
}

registerable_tuples!(T1);
registerable_tuples!(T1, T2);
registerable_tuples!(T1, T2, T3);
registerable_tuples!(T1, T2, T3, T4);
registerable_tuples!(T1, T2, T3, T4, T5);
registerable_tuples!(T1, T2, T3, T4, T5, T6);
registerable_tuples!(T1, T2, T3, T4, T5, T6, T7);
registerable_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
registerable_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
