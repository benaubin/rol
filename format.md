A database consists of multiple components


Structure:

```
db/checkpoint-a
db/checkpoint-b
db/log/[N]
```


## Checkpoint files

Written to two alternating locations.
We pick the checkpoint with the highest tail index.

```
Checkpoint := {
    # Only fully committed pages are included in checkpoint files
    Magic       := Int
    Checksum    := CRC32
    Log File    := Int
       # integer key to a log
    Stale size  := Int
       # how much space in the log is stale and could be
       # recovered by a snapshot
    Tail        := (Log Offset, Idx)
       # the last commited log entry referenced in this checkpoints
    OID mapping := [oid] -> [offset]
       # if (offset == 0), the oid is free
}
```

## Log files

Addressed by an integer key. The active log is the one referenced by the active
checkpoint.

```
Log Header := {
    Magic         := Int
    First Index   := Int
}

Log Entry := {
    checksum := CRC32
    term := Int64
       If 0, this is a compaction, and does not increment the log index.
       A compaction may not change the state of any object, and can be done
       by any node.
    operations_count := Int
    operations := Operation[]
}

Operation := { oid, prev_offset, data_len, data }
    An operation to an object
    PUT: prev_offset == 0
        Insert or replace the log entry.
    DELETE: prev_offset == MAX
        Allow the object id to be reused.
    PATCH: 0 < prev_offset < MAX
        Modify the object with the given `data` as a change delta.

```

We can snapshot a log, monotonically increasing the integer key (filename) of the log,
and copying log entries into the log (recommended to do this by compaction).
We can only compact objects to their state as of `first index` stored in the header.
Log entries after `first index` should be copied from the source log.

