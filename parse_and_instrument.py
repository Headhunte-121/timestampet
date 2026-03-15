import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Find every #[tauri::command] followed by #[tracing::instrument(...)] followed by the function signature
    # We will replace the #[tracing::instrument] line entirely

    # We use a positive lookahead to find the function signature below the instrument macro
    pattern = re.compile(
        r'(#\[tauri::command\]\s*)'
        r'(#\[tracing::instrument.*?\]\s*)'
        r'(pub (?:async )?fn \w+\s*\((.*?)\).*?\{)',
        re.DOTALL
    )

    def replacer(match):
        prefix = match.group(1)
        # old_instrument = match.group(2)
        func_sig_line = match.group(3)
        args_str = match.group(4)

        # Extract argument names
        # e.g., 'request_id: String, state: tauri::State<'_, AppState>' -> ['request_id', 'state']
        args = []
        for arg in args_str.split(','):
            arg = arg.strip()
            if arg:
                name = arg.split(':')[0].strip()
                # Remove mut
                name = name.replace('mut ', '').strip()
                args.append(name)

        # What to skip? Any argument that is complex or massive (app, state, window, app_handle, settings)
        skip_list = [a for a in args if a in ('state', 'app', 'app_handle', 'window', 'settings')]

        if skip_list:
            new_instrument = f'#[tracing::instrument(level = "debug", skip({", ".join(skip_list)}))]\n'
        else:
            new_instrument = '#[tracing::instrument(level = "debug")]\n'

        return f'{prefix}{new_instrument}{func_sig_line}'

    new_content = pattern.sub(replacer, content)

    with open(filepath, 'w') as f:
        f.write(new_content)

process_file('watchmark-tauri/src-tauri/src/commands.rs')
process_file('watchmark-tauri/src-tauri/src/vlc.rs')
