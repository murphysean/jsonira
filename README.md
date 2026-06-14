# Jsonira 🚀

A flexible, Jira-inspired task management system built with Rust, Axum, and SQLite.

## 📖 The Story Behind Jsonira

This project started as a personal journey to explore the intersection of Rust, SQLite, and modern web technologies. More importantly, it was built to support my family—specifically to help my son, who has ADHD, with executive function and management. 

The goal was to transform the chore of organizing into something fun, creating a "family board" where we could track points and manage tasks together. For a while, this lived on a locally hosted server in our home and worked incredibly well for us.

I'm uploading this now as a personal milestone. This entire project was built before the explosion of LLMs and AI coding assistants; it represents a period of "cranking through it" manually, solving the puzzles, and building the architecture from the ground up. It's a snapshot of a project that served a real purpose for my family and a great learning experience in Rust.

## ✨ Features

- **Creative Storage**: Uses a JSON-blob approach within SQLite for maximum schema flexibility.
- **Task Management**: Full lifecycle tracking including priorities, states, and tags.
- **Permissions**: Implements a flexible "Circle" based access control system.
- **Audit Trail**: Comprehensive history of actions for every task.
- **Modern Stack**: Powered by `Axum`, `Tokio`, and `Serde`.

## 🛠️ Tech Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: SQLite (with JSON extensions)
- **Async Runtime**: Tokio

## 🚀 Getting Started

### Prerequisites
- Rust (latest stable)
- SQLite3

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/murphysean/jsonira.git
   cd jsonira
   ```
2. Build and run:
   ```bash
   cargo run
   ```

## 📄 License
This project is licensed under the MIT License.
