class OneilError(Exception):
    def __init__(self, notes: list[str] = []):
        self.notes_ = notes

    def kind(self) -> str:
        raise NotImplementedError("Subclasses must implement this method")
    
    def context(self) -> str | None:
        raise NotImplementedError("Subclasses must implement this method")
    
    def message(self) -> str:
        raise NotImplementedError("Subclasses must implement this method")

    def notes(self) -> list[str]:
        return self.notes_

    def with_note(self, note: str):
        self.notes_.append(note)
        return self

    def __str__(self):
        if self.context() != None:
            return f"{self.kind()} {self.context()}: {self.message()}"
        else:
            return f"{self.kind()}: {self.message()}"
        

def add_trace(function):
    def wrapper(*args, **kwargs):
        try:
            return function(*args, **kwargs)
        except OneilError as e:
            raise e.with_note(f"In {function.__name__}")
    return wrapper
