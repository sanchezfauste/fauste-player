# Données et sauvegardes

## Emplacement des fichiers {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Paramètres (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlists et session | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache d'analyse | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Journaux et rapports de plantage | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Les chemins Linux suivent les variables XDG (`XDG_CONFIG_HOME`, etc.) quand
elles sont définies.

## Mode portable {#portable-mode}

Définissez la variable d'environnement `FAUSTE_HOME` sur un dossier, et tout
est conservé là à la place, dans `config/`, `data/`, `cache/` et `logs/`.
C'est utile sur une clé USB, ou pour garder des configurations séparées côte
à côte.

## Fichiers {#files}

| Fichier | Contenu |
|---|---|
| `config.json` | Paramètres, sorties, seuils d'analyse, réglages avancés |
| `playlists.json` | Playlists, pistes, marques de lecture, marqueurs manuels |
| `session.json` | Pour chaque lecteur : playlist affichée, pistes en cours et suivante, mode, position, volume, largeurs des colonnes (par colonne) |

## Sauvegarde automatique et copies de sécurité {#autosave-and-backups}

- Les modifications sont enregistrées environ une seconde après avoir eu
  lieu. Tant que quelque chose joue, la session (les positions) est
  actualisée au même rythme.
- Chaque enregistrement écrit un fichier temporaire puis remplace l'ancien,
  si bien qu'une coupure de courant ne laisse jamais un fichier à moitié
  écrit.
- Les versions précédentes sont conservées sous les noms `.bak1`, `.bak2` et
  `.bak3`.
- Si un fichier ne peut pas être lu, la plus récente bonne copie de sécurité
  est utilisée. Le fichier illisible est conservé, renommé
  `*.corrupt-<time>`, pour examen. L'application démarre toujours.

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
