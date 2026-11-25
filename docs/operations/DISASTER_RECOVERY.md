# Disaster recovery

Target local recovery point 24 h and recovery time 2 h, subject to drills. Snapshot through SQLite's backup API or a documented quiescent procedure. Use that snapshot to enumerate referenced blobs, ensure retention through copy, verify hashes, and manifest the metadata revision and digest set. Avoid copying a single active WAL-mode database file. [SQLite backup API](https://www.sqlite.org/backup.html)

Restore into a fresh directory. Check database integrity, event sequence and reference closure; verify every copied blob and rebuild projections. Run a read-only report before resuming work. Quarantine incompatible or partial copies and preserve original data. Record measured times and data loss. [Quality gates](../testing/QUALITY_GATES.md)
