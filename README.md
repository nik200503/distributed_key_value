# RustKV: Distributed Key-Value Store

**RustKV** is a high-performance, distributed NoSQL database written in Rust. It supports Leader-Follower replication, persistent storage (Log-Structured), and an HTTP API Gateway with connection pooling.

## 🏗 Architecture

```mermaid
graph TD
    User((User)) -->|HTTP| Gateway[API Gateway]
    
    subgraph "RustKV Cluster"
        Gateway -->|TCP Pooled| Leader[Leader Node :4000]
        Leader -->|Replication| Follower[Follower Node :4001]
        
        Leader <-->|Disk I/O| Disk1[(WAL Log)]
        Follower <-->|Disk I/O| Disk2[(WAL Log)]
    end
    
    


