# General messages
usage-prefix = UTILISATION:
commands-intro = Commandes:
commands-category = Commandes disponibles:
help-command = Afficher les informations d'aide
see-also = Voir aussi:

# Command descriptions
save-command-about = Sauvegarder les modifications actuelles dans une branche WIP
list-command-about = Lister toutes les branches WIP
delete-command-about = Supprimer une branche WIP
restore-command-about = Restaurer les modifications depuis une branche WIP

# Operation messages
saving-wip = Sauvegarde des modifications WIP...
created-branch = Branche '{ $name }' créée
staged-all-changes = Modifications indexées
committed-changes = Modifications validées
pushed-changes = Modifications poussées vers le dépôt distant
skipped-push-no-remote = Aucun dépôt distant configuré, envoi ignoré
switched-back = Retour à la branche '{ $name }'
delete-complete = Branche WIP supprimée avec succès
no-wip-branches = Aucune branche WIP trouvée pour l'utilisateur '{ $username }'
restoring-wip = Restauration des modifications depuis la branche '{ $name }'...
checked-out-branch = Branche '{ $name }' extraite
unstaged-changes = Modifications désindexées
stashed-changes = Modifications remisées
applied-stash = Modifications remisées appliquées
recreated-file-states = États des fichiers d'origine recréés
deleted-local-branch = Branche locale '{ $name }' supprimée
deleted-remote-branch = Branche distante '{ $name }' supprimée
restore-complete = Modifications de '{ $name }' restaurées avec succès
operation-cancelled = Opération annulée
branch-not-found = Branche '{ $name }' introuvable
branch-name = { $name }
wip-branch-created = Branche WIP '{ $name }' créée
wip-branch-deleted = Branche WIP '{ $name }' supprimée { $remote ->
    [true] (locale et distante)
    *[false] (locale uniquement)
}

# Dialog prompts
delete-branch-prompt = Supprimer cette branche ?
delete-all-prompt = Supprimer toutes les { $count } branches WIP ?
delete-remote-prompt = Supprimer aussi les { $count } branches distantes ?
select-branches-to-delete = Sélectionner les branches à supprimer :
selection-instructions = Espace pour sélectionner/désélectionner, Entrée pour confirmer
no-branches-selected = Aucune branche sélectionnée
selected-branches = Branches sélectionnées :
found-wip-branch = Branche WIP trouvée :
found-wip-branches = Branches WIP trouvées :

# Error messages
remote-delete-failed = Échec de la suppression de la branche distante '{ $name }' : { $error }

# Help messages
save-local-help = Ne pas pousser les modifications vers le dépôt distant
save-username-help = Spécifier un nom d'utilisateur personnalisé
save-datetime-help = Spécifier une date et une heure personnalisées
delete-branch-help = Nom de la branche à supprimer
delete-all-help = Supprimer toutes les branches WIP
delete-force-help = Ignorer la confirmation
delete-local-help = Supprimer uniquement les branches locales
restore-branch-help = Nom de la branche à restaurer
restore-force-help = Ignorer la confirmation
restore-autostash-help = Remiser et réappliquer automatiquement les modifications locales

# Stashing messages
stashing-existing-changes = Sauvegarde des modifications existantes...
restoring-existing-changes = Restauration des modifications existantes...
