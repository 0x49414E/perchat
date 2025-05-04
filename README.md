# Perchat

A lightweight, privacy-focused multiuser chat application with symmetric encryption over WebSockets.

## Overview

Perchat provides a secure communication platform where all messages are protected by strong encryption.

## Features

- **End-to-End Encryption**: All messages are symmetrically encrypted, ensuring only authorized participants can read the conversation
- **WebSocket-Based**: Real-time communication with minimal latency
- **Multiuser Support**: Designed for secure group conversations
- **Zero Knowledge**: The server never has access to unencrypted messages or encryption keys
- **Minimal Footprint**: No message storage on servers
- **Open Protocol**: Transparent encryption implementation you can audit

## Installing and running

```bash
git clone https://github.com/0x49414E/perchat.git

cd perchat

cargo run
```
