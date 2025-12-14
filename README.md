# LLM Inference Service
### Team member

- Dezhi Ren 1005736795
- Wanrou Zhang 1011562694

### Team contact email


## Motivation

The rapid adoption of large language models (LLMs) in real-world 
applications has heightened the demand for robust, low-latency inference
services. While Python-based solutions currently dominate the machine 
learning ecosystem, they often struggle with the performance overhead, 
memory inefficiency, and concurrency limitations inherent in dynamic 
languages—issues that become especially pronounced in production-scale 
LLM serving.

Rust presents a compelling alternative, offering exceptional performance,
memory safety, and fearless concurrency, which align closely with the 
requirements of high-throughput, low-latency LLM services. However, 
the current Rust landscape lacks a mature, feature-complete framework 
for LLM inference that is both production-ready and easy to integrate. 
Existing Rust implementations remain experimental or offer limited 
functionality, leaving a clear gap between Rust’s systems-level potential
and the practical needs of modern AI applications.

Our team is driven by the opportunity to bridge this gap by building a 
Rust-native LLM inference service that emphasizes efficiency, safety, and
extensibility. By tackling challenges such as multi-model orchestration, 
real-time streaming, and context-aware session management—all within a 
single Rust-based stack—we aim to deliver a solution that not only 
demonstrates Rust’s suitability for AI infrastructure but also provides 
a practical, open-source foundation for future AI-driven systems. This 
project represents a meaningful step toward consolidating performance 
and safety in LLM deployment, while also offering a technically enriching
experience in cutting-edge systems programming and AI integration.


## Objective

The objective of this project is to design and implement a high-performance, Rust-native LLM inference service that bridges the gap between Rust’s systems-level capabilities and the practical demands of modern AI applications. By leveraging Rust’s strengths in performance, memory safety, and concurrency, we aim to deliver a production-ready service that supports seamless multi-model orchestration, real-time token streaming, and interactive user sessions—all within a single, efficient runtime.

To realize this vision, the project will focus on delivering the following key features:

- Multi-Model Management – Enable simultaneous loading and management of multiple LLMs with support for dynamic model loading, unloading, and hot-swapping, allowing flexible and efficient resource utilization.

- Real-time Streaming Support – Implement server-sent events (SSE) to deliver token-by-token streaming responses, ensuring low-latency interaction for end-users.

- Chat Interface – Provide a web-based UI for intuitive interaction with the inference service, complete with conversation history management.

- File Parsing Support – Integrate asynchronous parsing of text files and code files, with the ability to manage uploaded files dynamically.

- Contextual Memory – Maintain session-aware memory for user prompts and uploaded documents, enabling coherent, context-rich dialogues.

- Session Management – Allow users to create, switch, and delete sessions, supporting isolated conversation contexts and efficient resource cleanup.

This project represents a novel contribution to the Rust ecosystem, where existing LLM serving solutions are either experimental or lack the comprehensive feature set required for practical deployment. By delivering a performant, safe, and extensible foundation for LLM inference, we aim to demonstrate Rust’s viability as a first-class language for AI infrastructure while providing a tangible, open-source tool that can be extended for more complex AI-driven applications.



## Key Features:

- Multi-Model Management

    - Load and manage multiple LLMs simultaneously

    - Dynamic model loading/unloading

    - Model hot-swapping

- Real-time Streaming Support

    - Server-sent events (SSE) for token-by-token streaming

- Chat Interface

    - Web-based UI to interact with the inference service

    - Conversation history management

- File Parsing support

    - Support parsing text content in docx, pdf and txt file
    - Support parsing code file such as .py, .cpp, .rs, etc.
    - Async parsing for each file uploaded

    - Ability to remove any uploaded file

- Contextual Memory

    - Support contextual memory for user prompt and files

- Session Management

    - Enable user to create, switch session and delete session

## User's Guide

## Reproducibility Guide

## Contributions

### Dezhi Ren
- Software structure design
- Backend
    - Main entrance
    - Http handler layer
    - File parser
    - Session management
    - Contextual memory 
    - Unit test
- Frontend React interface
- Presentation

### Wanrou Zhang
- Software structure design
- LLM research and selection
- Backend
    - Http handler layer
    - LLM management
    - Unit test
- Presentation and Demo
- User Guide
- Reproducibility QA

## Conclusion
This project underscores Rust's compelling advantages as a high-performance, thread-safe, and low-latency language for building AI inference services, offering clear benefits over existing Python-based frameworks and positioning itself as a foundational technology for next-generation AI infrastructure. At the same time, we recognize that Rust remains an evolving ecosystem with a still-maturing set of libraries for AI/ML, highlighting the need for broader community contribution and engagement to expand its capabilities. Throughout development, we also learned that delivering a performant LLM inference service depends on multiple factors—including GPU hardware, model architecture, and model size—requiring thorough upfront research and planning. Importantly, the current Rust landscape still lacks comprehensive support for loading and serving diverse AI models, which necessitated careful evaluation of available crates and a deeper understanding of model structures before implementation. In conclusion, while Rust presents a powerful and safe pathway for efficient AI serving, its full potential will be realized through continued ecosystem development, informed architectural choices, and active community collaboration.

## References 
This project uses the open-source MistralRS crate (MIT License):
 -- MistralRS GitHub: https://github.com/EricLBuehler/mistral.rs
We thank the MistralRS contributors for providing an efficient Rust-based LLM runtime.


# Video Slide Presentation

### Slide link
https://docs.google.com/presentation/d/1N8ETdvKPDZ93osGocNBkeiM7IdxmCsaoLbP2KXV4Hso/edit?usp=sharing

### Video link


# Video Demo

### link