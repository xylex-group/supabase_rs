# Contributing to supabase_rs

Thank you for your interest in contributing to `supabase_rs`! This guide will help you get started with development, testing, and submitting contributions.

## Project Vision

`supabase_rs` aims to provide a comprehensive, type-safe, and performant Rust SDK for Supabase. We prioritize:

- **Developer Experience**: Intuitive APIs that feel natural in Rust
- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Performance**: Efficient query building and HTTP request handling
- **Reliability**: Comprehensive error handling and testing
- **Documentation**: Clear, comprehensive documentation with examples

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Supabase Project**: For integration testing
- **Git**: For version control

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/supabase_rs.git
   cd supabase_rs
   ```

2. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your Supabase credentials for testing
   ```

3. **Install Dependencies**
   ```bash
   cargo check
   ```

4. **Run Tests**
   ```bash
   # Unit tests (no network required)
   cargo test unit_

   # Integration tests (requires Supabase credentials)
   cargo test

   # All tests with output
   cargo test -- --nocapture
   ```

## Project Structure

```text
supabase_rs/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── insert.rs           # Insert operations
│   ├── update.rs           # Update and upsert operations
│   ├── select.rs           # Query execution
│   ├── delete.rs           # Delete operations
│   ├── query_builder/      # Fluent query building
│   │   ├── builder.rs      # QueryBuilder implementation
│   │   ├── filter.rs       # Filter operations
│   │   └── sort.rs         # Sorting operations
│   ├── storage/            # File operations (feature-gated)
│   ├── graphql/            # GraphQL support (experimental)
│   ├── errors.rs           # Error types and handling
│   └── request/            # HTTP request utilities
├── tests/                  # Integration tests
└── docs/                   # Additional documentation
```

## Development Guidelines

### Code Style

- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Run `cargo clippy` and address all warnings
- **Documentation**: Document all public APIs with examples
- **Error Handling**: Use structured error types with helpful messages

### Documentation Standards

1. **Module Documentation**: Each module should have comprehensive docs explaining:
   - Purpose and scope
   - Core features and capabilities
   - Usage examples
   - Performance considerations
   - Error handling patterns

2. **Function Documentation**: All public functions must include:
   - Clear description of functionality
   - Parameter descriptions with types
   - Return value documentation
   - At least one usage example
   - Error conditions and handling

3. **Example Requirements**:
   - Use `rust,no_run` for examples that require external setup
   - Include complete, runnable examples when possible
   - Show error handling patterns
   - Demonstrate best practices

### Testing Requirements

1. **Unit Tests**: Test individual components in isolation
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_query_building() {
           // Test query construction without network calls
       }
   }
   ```

2. **Integration Tests**: Test against live Supabase instances
   ```rust
   #[tokio::test]
   async fn test_user_crud() -> Result<(), String> {
       // Test complete CRUD operations
   }
   ```

3. **Performance Tests**: Benchmark critical operations
   ```rust
   #[tokio::test]
   async fn test_bulk_insert_performance() {
       // Measure performance of bulk operations
   }
   ```

## Making Changes

### Feature Development

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement Changes**
   - Write code following project conventions
   - Add comprehensive documentation
   - Include relevant tests

3. **Test Thoroughly**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Update Documentation**
   - Update README.md if adding new features
   - Add examples to module documentation
   - Update CHANGELOG.md

### Bug Fixes

1. **Reproduce the Issue**
   - Create a test that demonstrates the bug
   - Understand the root cause

2. **Implement Fix**
   - Make minimal changes to fix the issue
   - Ensure no regressions are introduced

3. **Verify Fix**
   - Ensure the test now passes
   - Run full test suite

### Documentation Improvements

1. **Identify Gaps**
   - Look for undocumented public APIs
   - Find examples that could be clearer
   - Check for outdated information

2. **Enhance Documentation**
   - Add missing documentation
   - Improve existing examples
   - Add troubleshooting guides

## Testing Strategy

### Test Categories

1. **Unit Tests** (`cargo test unit_`)
   - Test individual functions
   - Mock external dependencies
   - Fast execution, no network

2. **Integration Tests** (`cargo test`)
   - Test against live Supabase
   - Require environment variables
   - Test complete workflows

3. **Performance Tests**
   - Benchmark query performance
   - Memory usage validation
   - Scalability testing

### Test Environment

Set up your test environment:

```env
# .env file for testing
SUPABASE_URL=https://your-test-project.supabase.co
SUPABASE_KEY=your-test-key
SUPABASE_RS_NO_NIGHTLY_MSG=true
```

### Writing Good Tests

```rust
#[tokio::test]
async fn test_insert_and_retrieve() -> Result<(), String> {
    let client = create_test_client();
    
    // Test data
    let test_user = json!({
        "name": "Test User",
        "email": "test@example.com"
    });
    
    // Insert
    let user_id = client.insert("users", test_user.clone()).await?;
    
    // Verify
    let users = client
        .select("users")
        .eq("id", &user_id)
        .execute()
        .await?;
    
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "Test User");
    
    // Cleanup
    client.delete("users", &user_id).await?;
    
    Ok(())
}
```

## Pull Request Process

### Before Submitting

1. **Check Requirements**
   - [ ] All tests pass (`cargo test`)
   - [ ] Code is formatted (`cargo fmt`)
   - [ ] No clippy warnings (`cargo clippy`)
   - [ ] Documentation is updated
   - [ ] CHANGELOG.md is updated (for features/fixes)

2. **Test Your Changes**
   ```bash
   # Run all checks
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

### PR Guidelines

1. **Title**: Use clear, descriptive titles
   - `feat: add bulk insert functionality`
   - `fix: resolve connection timeout in select operations`
   - `docs: improve GraphQL examples and error handling`

2. **Description**: Include:
   - What changes were made
   - Why the changes were necessary
   - How to test the changes
   - Any breaking changes

3. **Size**: Keep PRs focused and reasonably sized
   - One feature or fix per PR
   - Split large changes into multiple PRs

### Review Process

1. **Automated Checks**: CI will run tests and linting
2. **Code Review**: Maintainers will review your changes
3. **Feedback**: Address any requested changes
4. **Merge**: Approved PRs will be merged by maintainers

## Contribution Areas

### High-Priority Areas

1. **Core Functionality**
   - Improve query builder performance
   - Add missing CRUD operations
   - Enhance error handling

2. **Storage Operations**
   - Add file upload functionality
   - Implement streaming downloads
   - Add progress tracking

3. **GraphQL Stabilization**
   - Add mutation support
   - Implement subscription handling
   - Improve error reporting

4. **Documentation**
   - Add more comprehensive examples
   - Create tutorial guides
   - Improve API documentation

### Feature Requests

Before implementing new features:

1. **Check Issues**: Look for existing feature requests
2. **Discuss First**: Open an issue to discuss the feature
3. **Get Approval**: Wait for maintainer feedback
4. **Implement**: Follow the development process

## Bug Reports

### Reporting Bugs

1. **Search First**: Check if the bug is already reported
2. **Provide Details**: Include:
   - Rust version
   - Crate version
   - Operating system
   - Minimal reproduction case
   - Expected vs actual behavior

3. **Use Template**: Follow the bug report template

### Bug Report Template

```markdown
**Bug Description**
A clear description of what the bug is.

**Reproduction Steps**
1. Initialize client with...
2. Call method...
3. Observe error...

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]
- Crate version: [e.g., 0.4.14]
- Features enabled: [e.g., storage, rustls]

**Additional Context**
Any other relevant information.
```

## Resources

### Documentation

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Supabase Documentation](https://supabase.io/docs)
- [PostgREST API Reference](https://postgrest.org/en/stable/api.html)

### Tools

- [rustfmt](https://github.com/rust-lang/rustfmt) - Code formatting
- [clippy](https://github.com/rust-lang/rust-clippy) - Linting
- [cargo-doc](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) - Documentation generation

## Community

### Communication

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Pull Requests**: For code contributions

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow Rust community standards

##  Recognition

Contributors are recognized in:
- README.md contributors section
- CHANGELOG.md for significant contributions
- GitHub contributor graphs

Thank you for contributing to `supabase_rs`! Your efforts help make Rust development with Supabase better for everyone.
