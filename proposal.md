# Topic
### Team member

- Dezhi Ren 1005736795
- Wanrou Zhang 1011562694
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


## Objective and key features

The recent explosion in large language model (LLM) usage has created a growing need for efficient, reliable inference services. While Python dominates the ML ecosystem, Rust offers significant advantages in performance, memory safety, and concurrency that are particularly valuable for serving LLMs. Current Rust solutions in this space are either experimental or lack comprehensive feature sets, creating a gap between the performance potential of Rust and the practical needs of production LLM serving.

Our team is motivated to build this project because it addresses a real need in the Rust ecosystem while allowing us to work with cutting-edge AI technology. The challenge of building a high-performance inference service that can handle multiple models simultaneously while providing streaming responses is both technically satisfying and practically valuable. This project fills an important gap by providing a Rust-native solution that can serve as a foundation for more complex AI applications.

Key Features:

- Multi-Model Management

 - - Load and manage multiple LLMs simultaneously

- - Dynamic model loading/unloading

- - Model versioning and hot-swapping

- RESTful API Endpoints

- - Text completion endpoints with configurable parameters

- - Model information and health check endpoints

- - Batch processing support

- - Rate limiting and request queuing

- Real-time Streaming Support

- - Server-sent events (SSE) for token-by-token streaming

- - WebSocket support for bidirectional communication

- - Configurable chunk sizes and streaming delays

- Basic Chat Interface

- - Web-based UI to interact with the inference service

- - Conversation history and context management

- - Model selection and parameter tuning

- Performance Optimization

- - Async inference processing

- - Efficient memory management for large models

- - GPU acceleration support where available

Novelty: While several LLM serving solutions exist in Python, a Rust-native implementation focusing on performance, safety, and ease of use represents a gap in the current ecosystem. Our solution will leverage Rust's strengths to provide better resource utilization and lower latency compared to existing alternatives.

## Tentative plan

- Phase 1: Foundation (Week 1-2)

- - Set up project structure and dependencies

- - Implement basic model loading using Candle framework

- - Create core inference engine with blocking text generation

- - Build basic health check and model info endpoints using Axum

- Phase 2: API Development (Week 3-4)

- - Design and implement complete REST API specification

- - Add request validation and error handling

- - Implement batch processing capabilities

- - Add rate limiting and basic security measures

- Phase 3: Streaming & Advanced Features (Week 5-6)

- - Implement server-sent events for streaming responses

- - Add WebSocket support for real-time interaction

- - Develop model management system (load/unload multiple models)

- - Optimize memory usage and inference performance

- Phase 4: UI & Polish (Week 7-8)

- - Build basic web chat interface

- - Add comprehensive testing and documentation

- - Performance benchmarking and optimization

- - Final integration and deployment preparation

- Responsibilities:

- - Backend development (model inference, API server)

- - Frontend development (chat interface)

- - System architecture and design

- - Testing, documentation, and deployment

The project is designed to be completed within the timeframe by focusing on core functionality first and iteratively adding advanced features. The modular architecture allows for parallel development of different components, and the use of mature Rust crates (Candle for inference, Axum for web services) reduces implementation risk.