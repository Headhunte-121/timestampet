with open('timemark/main.py', 'r') as f:
    content = f.read()

# I hardcoded 2024 as the fallback for "Present".
# Update it to check the current system year dynamically.
content = content.replace(
    'if min_year and max_year == "2024": # Simple fallback or just let it use max year',
    'import datetime\n            curr_year = str(datetime.datetime.now().year)\n            if min_year and max_year == curr_year: # Dynamically use current system year'
)

with open('timemark/main.py', 'w') as f:
    f.write(content)
