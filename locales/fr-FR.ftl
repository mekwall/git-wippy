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
applied-changes = Modifications de la branche WIP appliquées
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
save-message-help = Spécifier un message personnalisé pour le commit WIP
list-all-help = Afficher les branches WIP de tous les utilisateurs

# Stashing messages
stashing-existing-changes = Sauvegarde des modifications existantes...
restoring-existing-changes = Restauration des modifications existantes...
restore-local-changes-error = Vous avez des modifications locales. Veuillez les valider ou les remiser, ou utiliser --autostash
restore-stash-failed = Échec de la mise en stash des modifications
restore-stash-list-failed = Échec de la récupération de la liste des stashes
restore-stash-not-found = Impossible de trouver un stash nommé : { $name }
restore-temp-branch-create-failed = Échec de la création de la branche temporaire
restore-source-branch-switch-failed = Échec du retour à la branche source
restore-temp-branch-delete-failed = Échec de la suppression de la branche temporaire
restore-stash-drop-failed = Échec de la suppression du stash
restore-existing-changes-failed = Échec de la restauration des modifications existantes : { $error }
restore-stashed-apply-failed = Échec de l'application des modifications remisées : { $error }
restore-select-wip-prompt = Sélectionnez une branche WIP à restaurer
restore-select-wip-failed = Échec de la sélection d'une branche WIP
