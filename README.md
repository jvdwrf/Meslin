# Meslin
Meslin is a Rust library offering ergonomic wrappers for channels like `mpmc` and `broadcast`. It's designed to ease the creation of actor systems by adding user-friendly features, without being tied to any specific runtime. This makes it compatible with various runtimes such as `tokio`, `smol`, or `async-std`.

## Purpose and Design
The primary goal of Meslin is to provide a framework for developing actor systems in Rust. It intentionally steers clear of incorporating supervisory functions or other complex features, focusing instead on simplicity and non-interference.

### Core Components
Meslin is built around three key elements:
1. **Messages**: Defined with input and output parameters, messages facilitate sending operations using only the input. This design promotes the creation of messages that expect responses, enhancing their usability.
2. **Protocols**: An actor in Meslin establishes a protocol. This protocol, detailing the acceptable message types, is defined by implementing `From<M>` and `TryInto<M>` traits.
3. **Senders**: These are responsible for defining the delivery mechanism of a protocol to the actor.

### Flexibility and Extensibility
One of the library's strengths lies in its separation of concerns. This separation not only streamlines the development process but also enables the easy integration and customization of messages, protocols, and senders. For instance, swapping an `mpmc` channel for a `priority` channel is straightforward.

### Dynamic Senders
A unique feature of Meslin is the transformation of senders into dynamic senders. This process converts the sender into a trait-object, facilitating the storage of different sender types in the same data structure, like `Vec<T>`. For example, if you have an `mpmc::Sender<ProtocolA>` and a `broadcast::Sender<ProtocolB>`, both accepting messages `Msg1` and `Msg2`, they can be converted into `DynSender<Accepts![Msg1, Msg2]>`. This dynamic sender then implements `Sends<Msg1> + Sends<Msg2>`, allowing for versatile storage solutions.

### Zero-cost
Meslin is designed with a zero-cost abstraction principle in mind, ensuring that its ease of use and flexibility don't compromise performance. When not using any dynamic features of the library, Meslin does not add any additional runtime overhead compared to hand-written equivalents.

## Cargo features
- Default: `["derive", "request", "mpmc", "broadcast", "priority"]`
- Non-default: `["watch"]`
