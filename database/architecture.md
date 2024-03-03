# Architecture

According to *Architecture of a Database System*, a common DBMS has following components:

1. Client communications manager: `uranus-s`, `uranus-c`

2. Relational query processor: `uranus-p`

3. Transactional Storage manager: `uranus-kv`

4. Process Manager: inside `uranus-s`

5. Shared components and utilities: inside `uranus-p`
