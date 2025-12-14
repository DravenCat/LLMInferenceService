# LLM Inference Service
### Team member

- Dezhi Ren 1005736795
- Wanrou Zhang 1011562694

### Team contact email


## Motivation

The recent explosion in large language model (LLM) usage has created
a growing need for efficient, reliable inference services. While
Python dominates the ML ecosystem, Rust offers significant advantages
in performance, memory safety, and concurrency that are particularly
valuable for serving LLMs. Current Rust solutions in this space are
either experimental or lack comprehensive feature sets, creating a
gap between the performance potential of Rust and the practical needs
of production LLM serving.

Our team is motivated to build this project because it addresses a
real need in the Rust ecosystem while allowing us to work with
cutting-edge AI technology. The challenge of building a
high-performance inference service that can handle multiple models
simultaneously while providing streaming responses is both
technically satisfying and practically valuable. This project fills
an important gap by providing a Rust-native solution that can serve
as a foundation for more complex AI applications.


## Objective

The recent explosion in large language model (LLM) usage has created a growing need for efficient, reliable inference services. While Python dominates the ML ecosystem, Rust offers significant advantages in performance, memory safety, and concurrency that are particularly valuable for serving LLMs. Current Rust solutions in this space are either experimental or lack comprehensive feature sets, creating a gap between the performance potential of Rust and the practical needs of production LLM serving.

Our team is motivated to build this project because it addresses a real need in the Rust ecosystem while allowing us to work with cutting-edge AI technology. The challenge of building a high-performance inference service that can handle multiple models simultaneously while providing streaming responses is both technically satisfying and practically valuable. This project fills an important gap by providing a Rust-native solution that can serve as a foundation for more complex AI applications.

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

    - Async parsing for each file uploaded

    - Ability to remove any uploaded file

- Contextual Memory


- Session Management

Novelty: While several LLM serving solutions exist in Python, a Rust-native implementation focusing on performance, safety, and ease of use represents a gap in the current ecosystem. Our solution will leverage Rust's strengths to provide better resource utilization and lower latency compared to existing alternatives.


## Contributions

### Dezhi Ren
- Software structure design
- Backend
    - Main entrance
    - Http handler layer
    - File parser layer
    - Contextual memory and session 
- Frontend React interface

### Wanrou Zhang
- LLM research and selection
- Backend
    - Http handler layer
    - LLM manager layer
- User Guide
- Reproducibility QA

## Conclusion
This version of the project integrates an LLM inference service using MistralRS.
The basic functionality has been implemented and tested.

## References 
This project uses the open-source MistralRS crate (MIT License):
 -- MistralRS GitHub: https://github.com/EricLBuehler/mistral.rs
We thank the MistralRS contributors for providing an efficient Rust-based LLM runtime.


# Video Slide Presentation

### link


# Video Demo

### link