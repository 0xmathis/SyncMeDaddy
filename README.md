# SyncMeDaddy

## How it works

### Connection
```mermaid
sequenceDiagram
    Client->>Server: Connect: "username"
    Server->>Client: Connect: "OK"
```

### Update
```mermaid
sequenceDiagram
    Client->>Server: UpdateRequest: state
    Server->>Client: Update: diff_state
    loop For each file to upload
        Client->>Server: Upload: file
    end
    Client->>Server: Updated
    loop For each file to download
        Server->>Client: Download: file
    end
    Server->>Client: Updated
```
