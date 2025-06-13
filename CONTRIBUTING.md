# Contributing to Geotiles

Thank you for your interest in contributing to Geotiles! This document outlines the process for contributing to this geodesic polyhedra library.

## 🤝 Ways to Contribute

- **Bug Reports**: Found an issue? Let us know!
- **Feature Requests**: Have an idea? We'd love to hear it!
- **Code Contributions**: Bug fixes, new features, optimizations
- **Documentation**: Improvements to docs, examples, tutorials
- **Testing**: More test cases, edge case coverage
- **Performance**: Benchmarks, optimizations, profiling

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A GitHub account

### Setting Up Development Environment

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/geotiles.git
   cd geotiles
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/originalowner/geotiles.git
   ```
4. **Install dependencies** and run tests:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

## 📝 Development Workflow

### Before You Start

1. **Check existing issues** to avoid duplicate work
2. **Create an issue** for significant changes to discuss the approach
3. **Keep changes focused** - one issue per pull request

### Making Changes

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write your code** following our style guidelines (see below)

3. **Add tests** for new functionality:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_your_new_feature() {
           // Your test here
       }
   }
   ```

4. **Update documentation** as needed:
   - Add doc comments for public APIs
   - Update README if adding major features
   - Add examples for complex functionality

5. **Run the full test suite**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt
   cargo doc --no-deps
   ```

### Submitting Changes

1. **Commit your changes** with clear messages:
   ```bash
   git commit -m "Add feature: brief description
   
   Longer explanation of what this commit does and why.
   Fixes #123"
   ```

2. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Create a Pull Request** on GitHub with:
   - Clear title and description
   - Reference to related issues
   - Description of changes made
   - Any breaking changes noted

## 📋 Code Style Guidelines

### Rust Style

- **Follow `rustfmt`**: Run `cargo fmt` before committing
- **Use `clippy`**: Fix all warnings from `cargo clippy`
- **Meaningful names**: Use descriptive variable and function names
- **Documentation**: All public APIs must have doc comments

### Code Structure

```rust
/// Brief one-line description.
///
/// Longer description explaining the purpose, usage, and any important
/// details about the function or struct.
///
/// # Arguments
///
/// * `param` - Description of the parameter
///
/// # Returns
///
/// Description of what is returned
///
/// # Examples
///
/// ```rust
/// let result = your_function(42);
/// assert_eq!(result, expected_value);
/// ```
///
/// # Panics
///
/// Describe when this function might panic (if applicable)
pub fn your_function(param: i32) -> i32 {
    // Implementation
}
```

### Testing Guidelines

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test public API behavior
- **Edge cases**: Test boundary conditions, empty inputs, etc.
- **Performance tests**: For algorithms with performance requirements
- **Property-based tests**: Consider using `quickcheck` for complex algorithms

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        let result = basic_function(input);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_edge_case_empty_input() {
        let result = function_with_empty(&[]);
        assert!(result.is_empty());
    }
    
    #[test]
    #[should_panic(expected = "Invalid input")]
    fn test_invalid_input_panics() {
        invalid_function(-1);
    }
}
```

### Documentation Standards

- **Public APIs**: Must have comprehensive doc comments
- **Examples**: Include usage examples in doc comments
- **Mathematical concepts**: Explain algorithms and formulas
- **Performance notes**: Document complexity where relevant
- **Safety**: Document unsafe code thoroughly

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run doctests
cargo test --doc
```

### Test Coverage

We aim for high test coverage. When adding new features:

1. **Add unit tests** for all new functions
2. **Add integration tests** for new public APIs
3. **Test error conditions** and edge cases
4. **Update existing tests** if changing behavior

### Performance Testing

For performance-critical code:

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn bench_subdivision_performance() {
        let start = Instant::now();
        let _result = expensive_operation();
        let duration = start.elapsed();
        
        // Assert reasonable performance bounds
        assert!(duration.as_millis() < 100);
    }
}
```

## 📚 Documentation

### API Documentation

- Use `///` for doc comments on public items
- Include examples that compile and run
- Explain parameters, return values, and behavior
- Document panics, errors, and safety considerations

### README Updates

When adding significant features:
- Update the feature list
- Add usage examples
- Update performance characteristics
- Add to the applications section if relevant

### Examples

Add examples to the `examples/` directory for:
- Common use cases
- Integration with popular frameworks
- Performance demonstrations
- Advanced features

## 🔍 Code Review Process

### For Contributors

- **Be responsive** to feedback
- **Make requested changes** promptly
- **Ask questions** if feedback is unclear
- **Keep discussions focused** on the code

### Review Criteria

Pull requests are evaluated on:

1. **Correctness**: Does the code work as intended?
2. **Testing**: Are there adequate tests?
3. **Documentation**: Is the code well-documented?
4. **Style**: Does it follow project conventions?
5. **Performance**: Are there any performance regressions?
6. **API Design**: Is the API intuitive and consistent?

## 🐛 Bug Reports

### Before Reporting

1. **Search existing issues** to avoid duplicates
2. **Try the latest version** to see if it's already fixed
3. **Create a minimal reproduction** case

### Bug Report Template

```markdown
## Bug Description
Brief description of the bug.

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What you expected to happen.

## Actual Behavior
What actually happened.

## Environment
- Geotiles version: 
- Rust version: 
- Operating system: 
- Additional context: 

## Minimal Code Example
```rust
// Code that reproduces the issue
```

## Additional Information
Any other relevant information, stack traces, etc.
```

## ✨ Feature Requests

### Before Requesting

1. **Check existing issues** and discussions
2. **Consider the scope** - does it fit the library's goals?
3. **Think about API design** - how would it work?

### Feature Request Template

```markdown
## Feature Description
Clear description of the proposed feature.

## Use Case
Why is this feature needed? What problem does it solve?

## Proposed API
How would users interact with this feature?

```rust
// Example of how the API might look
```

## Alternatives Considered
What other approaches did you consider?

## Implementation Notes
Any thoughts on how this might be implemented?
```

## 🎯 Specific Contribution Areas

### High-Priority Areas

- **Performance optimizations**: Subdivision algorithms, memory usage
- **Additional export formats**: STL, PLY, glTF support
- **Visualization tools**: Debug utilities, mesh validation
- **Integration examples**: More framework integrations
- **Mathematical accuracy**: Improved sphere projections

### Mathematical Improvements

- **Better face ordering**: Implement proper face sorting around vertices
- **Adaptive subdivision**: Quality-based subdivision criteria
- **Alternative projections**: Different sphere mapping algorithms
- **Precision handling**: Better floating-point precision management

### API Enhancements

- **Builder pattern**: Fluent API for configuration
- **Streaming generation**: Large meshes without full memory allocation
- **Parallel processing**: Multi-threaded subdivision
- **No-std support**: Embedded/constrained environment support

## 🔧 Development Tools

### Recommended Tools

- **IDE**: VS Code with rust-analyzer
- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Documentation**: `cargo doc --open`
- **Testing**: `cargo test`
- **Benchmarking**: Consider `criterion` for performance tests

### Useful Commands

```bash
# Check everything
cargo check && cargo test && cargo clippy && cargo fmt --check

# Generate documentation
cargo doc --no-deps --open

# Check for unused dependencies
cargo +nightly udeps

# Security audit
cargo audit
```

## 📄 License

By contributing to Geotiles, you agree that your contributions will be licensed under the same MIT License that covers the project.

## ❓ Questions?

- **Open an issue** for technical questions
- **Start a discussion** for design questions
- **Check existing issues** for common questions

Thank you for contributing to Geotiles! 🌐✨
