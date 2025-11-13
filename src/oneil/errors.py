class OneilError(Exception):
    def kind(self) -> str:
        raise NotImplementedError("Subclasses must implement this method")
    
    def context(self) -> str | None:
        raise NotImplementedError("Subclasses must implement this method")
    
    def message(self) -> str:
        raise NotImplementedError("Subclasses must implement this method")

    def notes(self) -> list[str]:
        if hasattr(self, "notes_"):
            return self.notes_
        else:
            return []

    def with_note(self, note: str):
        if hasattr(self, "notes_"):
            self.notes_.append(note)
        else:
            self.notes_ = [note]
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
