import os
import sys

# Ensure the parent directory is in the path
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

from timemark.main import App

if __name__ == "__main__":
    app = App()
    app.mainloop()
