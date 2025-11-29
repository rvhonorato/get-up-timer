# TODO

This that could be improved in this code

## Fix these first

- Writing state file every pooling interval is too much, only write on state change

## Performance

- Too many allocations in write_state_to_file()
- Cache formatted strings when state unchanged
- Move buffer allocation outside loops

## Security/Safety

- /tmp/user_state vulnerable to symlink attacks, use XDG_RUNTIME_DIR or ~/.cache (?)
- No file locking
- unsafe block needs safety comments
- Set explicit file permissions

## Error handling

- Everything panics instead of returning Results
- No recovery on write errors
- Should use thiserror or anyhow

## Code cleanup

- No tests (especially bad with unsafe code)
- Missing docs/comments
- State machine transitions undocumented
- User struct does too much, separate concerns
- Add proper logging instead of println/eprintln

## Features/Config

- Make timing constants configurable (CLI args or config file)
- Signal handling for clean shutdown
- Device filtering should be runtime option
- Better error messages
- Write a README
