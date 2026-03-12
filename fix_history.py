import re

with open('timemark/main.py', 'r') as f:
    content = f.read()

# Replace the naive legacy dt_local assignment with an aware one
old_block = """                if row['is_legacy']:
                    dt_local = datetime.strptime(row['timestamp'], '%Y-%m-%d %H:%M:%S')"""

new_block = """                if row['is_legacy']:
                    # Force naive datetime to be offset-aware (assuming local or UTC, assigning local timezone offset)
                    dt_local = datetime.strptime(row['timestamp'], '%Y-%m-%d %H:%M:%S').astimezone()"""

content = content.replace(old_block, new_block)

with open('timemark/main.py', 'w') as f:
    f.write(content)
