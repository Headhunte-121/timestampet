import os
import re

components_to_instrument = [
    "watchmark-tauri/src/components/Dashboard.tsx",
    "watchmark-tauri/src/components/History.tsx",
    "watchmark-tauri/src/components/Library.tsx",
    "watchmark-tauri/src/components/MediaDetails.tsx"
]

for file in components_to_instrument:
    with open(file, 'r') as f:
        content = f.read()

    # Add logger import if not present
    if "import { logger }" not in content:
        content = content.replace('import', 'import { logger } from "../utils/logger";\nimport', 1)

    with open(file, 'w') as f:
        f.write(content)
