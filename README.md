# Simulated Transaction Engine
The idea was to write a quick and naive implementation of a financial
transaction engine using the actor model. For this simulated scenario the
program takes as input a CSV file representing a series of transactions then
writes to `STDOUT` the account info for each client after processing.

While I am aware of the `Actix` crate for an actor model framework in Rust, I
decided to implement two extremely basic actors using what `Tokio` has to
offer for the sake of simplicity. The majority of the decisions made for this
small project were for the sake of simplicity and readability. Examples:

* Parsing the CSV file synchronously instead of asynchronously. Since I decided
	to use `Tokio` as the asynchronous runtime, using the `csv-async` crate
	would have required importing yet another crate to add more code to ensure
	compatibility between `tokio::io::AsyncRead` and `futures_io::AsyncRead`.
* The use of the actor model itself. This type of concurrent model allows for
	message-passing concurrency which results in a lockless design and
	restricts state mutation to processes which have exclusive ownership of
	some state. No locks to synchronize access to shared mutable state means
	less code to read. Additionally, the code in this example heavily relies
	upon copy by value to remain thread-safe.
* Two actors to represent synchronized mutation of the "database tables", but
	no actor for handling concurrent transaction streams and processing them
	sequentially. The entirety of the logic in `pay_engine::process()`, and more,
	would exist in said actor. However, I decided 2 actor implementations was
	enough to express the intent of the design.
* Lack of extensibility of the actor implementations, lack of logging
	capability, and hardcoded persist logic. Not only is there already a lot
	of duplicate code within the implementations, ideally we'd want our actors
	to be generic and require the user to supply their own functions for
	persisting state, logging, etc. These structs could then be reused to spin
	up different servers and specify different `receiver` and `sender` types
	operating on different message types, etc.
* Returning `Option` types instead of `Result` types from the actor handlers,
	and actors themselves. There was no other reason to do this other than
	to save a bit of time and code for simplicity and readability. Comments
	in `pay_engine::process()` expand on this a little more though it's very
	easy to see the `Result` type offers more robust handling of fallible
	operations and allow cleaner control flow to log and perform subsequent
	logic.
* Use of `pay_engine::transaction::Transaction` type in this project instead of
	`pay_engine::messages::TransactionMsg`. I would prefer to use the latter to
	deserialize the CSV records into and use as the message type for
	message-passing to the absent "process actor" (in this case is the
	`pay_engine::process()` function). However, there doesn't seem to be any
	built in support for this type of deserialization with `csv` and `serde`.
	Therefore, in order to save some extra steps and code, I settled with the
	type the code is using for now.
* Absence of unit tests upon initial commit and cargo docs.

(More explanations are provided in a handful of small comments in the code.)

### Files
* account.rs: A struct that represents a client's account state. Includes a
	function for state transition.
* account\_registry.rs: An actor struct responsible for reading and writing
	to and from the data source containing all client accounts.
* actor.rs: Bare minimum traits for actor types to demonstrate the potential
	of extensibility and maintainability.
* engine.rs: The library entry point suitable only for this current simulated
	project.
* messages.rs: Message enum types to send between actors.
* runner.rs: Binary entry point.
* transaction.rs: A struct that represents a transaction's state.
* transaction\_registry.rs: An actor struct reponsible for reading and writing
	to and from the data source containing all processed transactions.

### Explanation
The bulk of the processing logic exists in `pay_engine::process()`.
`AccountRegistryHandle` and `TransactionRegistryHandle` are really only there
to handle lookup and persist operations on the stored client accounts and
transactions.

In `pay_engine::process()` the CSV is parsed with a call to
`pay_engine::parse_file()` which returns a `Vec<Transaction>`. Processing
continues by converting the collection of transactions to a
`tokio_stream::Stream` in order to process the stream of transaction messages
asynchronously.

The large `while let` that makes up the rest of the processing function applies
logic for each message with minimal error handling and logging. (A real
application would have much more robust error handling as mentioned in comments
in the code.)

When it is time to apply a change to a client account, the `apply()` function
is called on the account object that is to change and returns a **new** copy
of the account object with updated values. This new copy is then persisted to
a data source with a call to `AccountRegistryHandler.persist()`. If the call
was successful, the transaction is then persisted to a data source. In the
case of Dispute transactions, the transaction disputed is also updated and
stored.

Upon completion the simulated "request thread" (`process()`) joins with the
parent "server" task (tokio runtime) and the returned `Result` is mapped to
another `Result` holding a collection of all accounts. The final `Result`
is then mapped accordingly to a call to either `success()` or `fail()` which
handles the remaining execution for the rest of the binary. That is, it prints
the client accounts to `STDOUT` on success or an error message to `STDERR` on
failure.
