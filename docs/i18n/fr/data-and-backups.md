# Données et sauvegardes

## Emplacement des fichiers {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Paramètres (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Bibliothèque, playlists, cartouches et session (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache d'analyse | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Journaux et rapports de plantage | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Les chemins Linux suivent les variables XDG (`XDG_CONFIG_HOME`, etc.) quand
elles sont définies.

Le snap conserve plutôt ses fichiers dans son propre dossier : les réglages
dans `~/snap/fauste-player/current/.config/fausteplayer/`, la base de données
et les journaux dans `~/snap/fauste-player/current/.local/share/fausteplayer/`,
et le cache d'analyse dans `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` les supprime aussi (il garde un instantané
pendant un certain temps ; `--purge` non).

## Mode portable {#portable-mode}

Définissez la variable d'environnement `FAUSTE_HOME` sur un dossier, et tout
est conservé là à la place, dans `config/`, `data/`, `cache/` et `logs/`.
C'est utile sur une clé USB, ou pour garder des configurations séparées côte
à côte.

## Fichiers {#files}

| Fichier | Contenu |
|---|---|
| `config.json` | Paramètres, sorties, seuils d'analyse, réglages avancés |
| `fauste.db` | La bibliothèque (pistes, leurs catégories, marqueurs manuels, tags, loudness), les playlists avec leurs marques de lecture, les pages de cartouches, les rotations de l'automatisation, la grille et le journal de diffusion (conservé `automation.as_run_keep_days` jours, 90 par défaut), et la session : pour chaque lecteur, la playlist affichée, les pistes en cours et suivante, le mode, la position, le volume et les largeurs des colonnes (par colonne), ainsi que la disposition de la fenêtre |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Copies de la base de données (voir ci-dessous) |
| `window.ron` | La taille et la position de la fenêtre principale, et si elle était agrandie (ordinateur seulement ; sous Wayland, le système choisit la position) |

## Sauvegarde automatique et copies de sécurité {#autosave-and-backups}

- Les modifications sont enregistrées environ une seconde après avoir eu
  lieu, et seul ce qui a changé est écrit. Tant que quelque chose joue, la
  session (les positions) est actualisée au même rythme.
- La base de données est écrite par transactions, si bien qu'une coupure de
  courant ne laisse jamais une modification à moitié enregistrée.
- Au premier enregistrement de chaque session, puis environ toutes les cinq
  minutes tant que des modifications sont enregistrées, l'application copie la
  base de données dans `fauste.db.bak1` et décale les copies plus anciennes
  vers `.bak2` et `.bak3`. Redémarrer plusieurs fois de suite chasse donc les
  sauvegardes plus anciennes.
- Si la base de données ne peut pas être lue, la plus récente bonne copie de
  sécurité est utilisée. Le fichier illisible est conservé, renommé
  `fauste.db.corrupt-<time>`, pour examen. Si toutes les copies échouent,
  l'application démarre à vide. Elle démarre toujours.
- Une copie inutilisable est conservée aussi : quand l'application démarre
  depuis une autre copie (ou une base vide), les copies inutilisables sont
  renommées `fauste.db.bak1.corrupt-<time>` (ou `.newer-<time>`) pour que la
  prochaine sauvegarde ne les chasse jamais.
- Si une autre copie de l'application utilise la base de données, celle-ci
  s'exécute sans enregistrer et le dit au démarrage.
- Une base de données écrite par une version plus récente de l'application
  n'est pas ouverte : elle est conservée sous le nom `fauste.db.newer-<time>`,
  et l'application démarre depuis la plus récente sauvegarde qu'elle peut
  lire, ou à vide s'il n'y en a pas, si bien qu'ouvrir les données avec une
  version plus ancienne ne les détruit jamais.
- `config.json` est enregistré comme auparavant : un fichier temporaire
  remplace l'ancien, et les versions précédentes sont conservées sous les noms
  `.bak1`, `.bak2` et `.bak3` à côté.
- Si la base de données ne peut pas du tout être utilisée pendant une session
  (par exemple, elle ne peut pas être renommée parce que le dossier est en
  lecture seule), l'application s'exécute sans enregistrer jusqu'au prochain
  démarrage, et le journal le dit.

## Playlists et session d'une version antérieure {#playlists-and-session-from-an-earlier-version}

Les versions antérieures conservaient les playlists dans `playlists.json`, les
pages de cartouches dans `carts.json` et la session dans `session.json`. Le
premier démarrage de cette version les lit (avec leurs copies `.bak`) une
fois et place tout dans `fauste.db` : la bibliothèque avec ses marqueurs,
tags et loudness, les playlists avec leurs marques de lecture, les pages de
cartouches et la session. Ensuite, les anciens fichiers sont renommés avec
`.imported` à la fin (`playlists.json.imported`,
`playlists.json.bak1.imported`, etc.). Ils ne sont plus utilisés et vous
pouvez les supprimer quand vous êtes sûr que tout est bien là.

Pour revenir à une version antérieure :

1. Fermez l'application.
2. Déplacez `fauste.db`, `fauste.db-wal`, `fauste.db-shm` et tous les
   `fauste.db.bak*` dans un dossier extérieur au dossier de données. Ne les
   supprimez pas.
3. Retirez `.imported` du nom des anciens fichiers.
4. Lancez la version plus ancienne.

Tout ce qui a été enregistré depuis la mise à jour reste uniquement dans le
dossier où vous avez déplacé les fichiers : les anciens fichiers ne
l'ont pas. Si vous mettez à jour de nouveau plus tard, les anciens fichiers
sont importés de nouveau, tant qu'aucun `fauste.db` ni `fauste.db.bak*` ne
se trouve dans le dossier de données.

Si un démarrage est interrompu pendant l'import, le suivant le termine. Si
l'un des anciens fichiers ne peut pas être lu (trop volumineux, ou écrit par
une version plus récente), l'application n'importe rien et s'exécute sans
enregistrer cette session, si bien que les anciens fichiers restent où ils
sont ; le journal nomme le fichier. Corrigez-le ou supprimez-le, puis
redémarrez. Une fois que `fauste.db` existe, les anciens fichiers JSON ne
sont plus jamais importés.

## Récupération après un plantage {#crash-recovery}

Après un plantage ou un redémarrage, chaque lecteur revient avec sa
playlist, ses pistes en cours et suivante, et sa position, mais **en pause ou
à l'arrêt**. Rien ne passe à l'antenne tout seul. Un lecteur qui se trouvait
tout à la fin de sa piste revient au début de cette piste, de sorte qu'appuyer
sur PLAY la lise au lieu de la terminer aussitôt.

## Modifier `config.json` à la main {#editing-configjson-by-hand}

Fermez d'abord l'application (si de l'audio est à l'antenne, elle demande
confirmation avant de fermer). Les valeurs inconnues ou hors plage sont
ramenées à la valeur valide la plus proche au chargement du fichier, et les
corrections sont consignées dans le journal. Les sections `limits` et `tuning`
contiennent des valeurs avancées (limites de ressources, temporisations du
moteur) qui ne figurent pas dans la fenêtre des Paramètres.
