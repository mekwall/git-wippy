# General messages
usage-prefix = VERWENDUNG:
commands-intro = Befehle:
commands-category = Verfügbare Befehle:
help-command = Hilfe anzeigen
see-also = Siehe auch:

# Command descriptions
save-command-about = Aktuelle Änderungen als WIP-Branch speichern
list-command-about = Alle WIP-Branches auflisten
delete-command-about = Einen WIP-Branch löschen
restore-command-about = Änderungen aus einem WIP-Branch wiederherstellen

# Operation messages
saving-wip = Speichere WIP-Änderungen...
created-branch = Branch '{ $name }' erstellt
staged-all-changes = Alle Änderungen gestaged
committed-changes = Änderungen committed
pushed-changes = Änderungen zum Remote gepusht
skipped-push-no-remote = Kein Remote-Repository konfiguriert, überspringe Push
switched-back = Zurück zum Branch '{ $name }' gewechselt
delete-complete = WIP-Branch erfolgreich gelöscht
no-wip-branches = Keine WIP-Branches für Benutzer '{ $username }' gefunden
restoring-wip = Stelle Änderungen von Branch '{ $name }' wieder her...
checked-out-branch = Branch '{ $name }' ausgecheckt
unstaged-changes = Änderungen unstaged
stashed-changes = Änderungen gestashed
applied-stash = Gestashte Änderungen angewendet
recreated-file-states = Ursprüngliche Dateizustände wiederhergestellt
deleted-local-branch = Lokaler Branch '{ $name }' gelöscht
deleted-remote-branch = Remote Branch '{ $name }' gelöscht
restore-complete = Änderungen von '{ $name }' erfolgreich wiederhergestellt
operation-cancelled = Operation abgebrochen
branch-not-found = Branch '{ $name }' nicht gefunden
branch-name = { $name }
wip-branch-created = WIP-Branch '{ $name }' erstellt
wip-branch-deleted = WIP-Branch '{ $name }' gelöscht { $remote ->
    [true] (lokal und remote)
    *[false] (nur lokal)
}

# Dialog prompts
delete-branch-prompt = Diesen Branch löschen?
delete-all-prompt = Alle { $count } WIP-Branches löschen?
delete-remote-prompt = Auch { $count } Remote-Branches löschen?
select-branches-to-delete = Branches zum Löschen auswählen:
selection-instructions = Leertaste zum Auswählen/Abwählen, Enter zum Bestätigen
no-branches-selected = Keine Branches ausgewählt
selected-branches = Ausgewählte Branches:
found-wip-branch = WIP-Branch gefunden:
found-wip-branches = WIP-Branches gefunden:

# Error messages
remote-delete-failed = Fehler beim Löschen des Remote-Branch '{ $name }': { $error }

# Help messages
save-local-help = Änderungen nicht zum Remote-Repository pushen
save-username-help = Benutzerdefinierten Benutzernamen angeben
save-datetime-help = Benutzerdefiniertes Datum und Uhrzeit angeben
delete-branch-help = Name des zu löschenden Branches
delete-all-help = Alle WIP-Branches löschen
delete-force-help = Bestätigung überspringen
delete-local-help = Nur lokale Branches löschen
restore-branch-help = Name des wiederherzustellenden Branches
restore-force-help = Bestätigung überspringen
restore-autostash-help = Lokale Änderungen automatisch stashen und wieder anwenden

# Stashing messages
stashing-existing-changes = Sichere bestehende Änderungen...
restoring-existing-changes = Stelle bestehende Änderungen wieder her...
