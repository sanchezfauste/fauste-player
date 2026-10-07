# Contrôle à distance

Fauste Player peut être lu et piloté par le réseau via une API HTTP, avec des
mises à jour en direct, et via OSC. Une page web, une application pour
téléphone, l'automatisation d'une station ou une surface de contrôle peuvent
les utiliser. Il est **désactivé** tant que vous ne l'activez pas, et au début
il ne répond que sur cet ordinateur.

## L'activer {#turning-it-on}

![Paramètres, À distance : l'API HTTP activée et à l'écoute sur cet ordinateur, et OSC désactivé](../../images/guide/settings-remote.png)

Ouvrez **Paramètres → À distance** et cochez **Autoriser le contrôle à
distance par HTTP** (ou **Autoriser le contrôle par OSC**). La ligne sous
chaque interrupteur indique si le serveur est à l'écoute, et où, ou pourquoi
il n'a pas démarré. Les changements s'appliquent aussitôt ; il n'y a pas
besoin de redémarrer. Un champ de texte (une adresse, le jeton, une liste)
s'applique quand vous le quittez, ouvrez une autre section ou fermez les
Paramètres ; une valeur pas encore valide conserve celle en cours d'usage, et
Esc annule ce que vous avez saisi.

Vous pouvez aussi modifier `config.json` pendant que Fauste Player est fermé
(voir [Données et sauvegardes](data-and-backups.md) pour son emplacement).
Dans l'objet `"config"`, définissez `remote.http.enabled` sur `true` :

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Il écoute sur `http://127.0.0.1:7380`. Le démarrer ne joue jamais rien ; seules
les requêtes agissent.

## Écouter sur le réseau du studio {#listening-on-the-studio-network}

Pour y accéder depuis d'autres ordinateurs, définissez `bind` sur `0.0.0.0`
(ou sur l'une des adresses de cet ordinateur) et définissez un **jeton** d'au
moins 16 caractères. Dans Paramètres → À distance, **Générer** crée un long
jeton aléatoire. Il est masqué jusqu'à ce que vous appuyiez sur **Afficher**,
et **Copier** le place dans le presse-papiers pour le client. Sans jeton, le
serveur refuse de démarrer, et le journal en donne la raison.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Les clients envoient le jeton sous la forme `Authorization: Bearer <token>`.
L'API n'est pas chiffrée. Gardez-la sur un réseau de studio de confiance, ou
placez-la derrière un proxy inverse avec HTTPS.

## Pages web {#web-pages}

Une page web servie depuis une autre adresse ne peut utiliser l'API que si
son origine (par exemple `https://studio.example`) figure dans
`cors_origins`. Les requêtes d'autres pages sont refusées, même sur cet
ordinateur, de sorte qu'une page que vous auriez ouverte par hasard ne peut
pas piloter le lecteur. `"*"` (toute origine) n'est accepté qu'avec un jeton.

## Ce qu'un client peut faire {#what-a-client-can-do}

Un client peut :

- lire les lecteurs, les playlists, les pistes (avec pochette et forme
  d'onde) et le mur de cartouches ;
- lire, mettre en pause, arrêter, faire un fondu, reprendre au début et
  revenir en arrière ;
- choisir l'entrée suivante (l'entrée à l'antenne aussi : elle est rejouée),
  pré-écouter, et se déplacer (sur un lecteur arrêté, un déplacement choisit
  où PLAY démarre l'entrée suivante, et un déplacement avant son cue-in
  démarre au cue-in) ;
- régler les volumes, les modes, le stop après la piste en cours, et les
  marques de répétition et d'arrêt après d'une entrée ;
- lancer, arrêter et pré-écouter des cartouches, et changer la page de
  cartouches affichée ;
- modifier : créer, renommer et supprimer des playlists ; ajouter une piste
  déjà chargée, et retirer, déplacer ou dupliquer des entrées ; créer,
  renommer, redimensionner et supprimer des pages de cartouches, et configurer
  une cartouche avec une piste chargée ; définir ou réinitialiser des
  marqueurs.

Retirer ce qui est à l'antenne est refusé, comme à l'écran. Les fichiers qui
ne sont pas encore chargés ne peuvent pas être ajoutés à distance : ils se
trouvent sur cet ordinateur, ajoutez-les donc d'abord ici.

Un bouton grisé à l'écran est aussi refusé à distance. La référence complète
se trouve dans [la documentation technique](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Essayez depuis un terminal :

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Mises à jour en direct {#live-updates}

Un client peut suivre les changements au fur et à mesure au lieu de
redemander sans cesse. `GET /api/v1/events` est un flux d'événements :
d'abord l'état complet, puis chaque changement de lecteur, de playlist, de
piste ou de mur de cartouches, et les temps de ce qui joue quelques fois par
seconde.

    curl -sN http://127.0.0.1:7380/api/v1/events

Une page web utilise `EventSource`. Les navigateurs ne peuvent pas y envoyer
le jeton dans un en-tête, il va donc dans l'adresse :
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC est le protocole habituel des surfaces de contrôle, des consoles
lumière et des logiciels de pilotage de spectacle. Activez-le avec
`remote.osc.enabled`. Il écoute sur le port UDP 7381 de cet ordinateur. Pour
accepter des paquets d'autres ordinateurs, définissez `remote.osc.bind` sur
`0.0.0.0` et listez leurs adresses ou sous-réseaux dans
`remote.osc.allowed_sources` (par exemple `"192.168.1.0/24"`). OSC n'a pas de
mot de passe, gardez-le donc sur un réseau de studio de confiance.

Les lecteurs sont numérotés 1, 2, 3… comme ils apparaissent à l'écran. Les
cartouches sont numérotées dans la page affichée.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Une surface qui veut afficher l'état (voyants, noms, comptes à rebours)
s'abonne, puis reçoit chaque valeur une fois et ensuite seulement ce qui
change. Quand un lecteur ou un bouton de cartouche disparaît (moins de
lecteurs, une page plus petite), ses adresses reçoivent une fois une valeur
vide, afin que la surface les efface. Elle doit se réabonner dans la minute
(`subscription_ttl_secs`) pour continuer à recevoir :

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Un abonné peut désigner n'importe quel port de sa propre adresse, et jusqu'à
`max_subscribers` sont conservés. Quiconque est autorisé à envoyer peut donc
aussi s'abonner. C'est une raison de plus de garder OSC sur un réseau de
confiance.

`oscsend` et `oscdump` sont fournis avec liblo (`liblo-tools` sous Debian et
Ubuntu). La liste complète des adresses se trouve dans
[la documentation technique](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Tous les réglages {#all-settings}

| Réglage | Défaut | Signification |
|---|---|---|
| `remote.http.enabled` | `false` | Activer l'API |
| `remote.http.bind` | `127.0.0.1` | Adresse d'écoute (une adresse IP) |
| `remote.http.port` | `7380` | Port (1024–65535) |
| `remote.http.token` | vide | Obligatoire au-delà de cet ordinateur ; au moins 16 caractères |
| `remote.http.cors_origins` | aucune | Origines web autorisées à appeler l'API |
| `remote.http.request_timeout_ms` | `10000` | Durée maximale d'une requête |
| `remote.http.max_body_bytes` | `65536` | Plus grand corps de requête |
| `remote.http.max_event_clients` | `16` | Flux d'événements en direct simultanés |
| `remote.osc.enabled` | `false` | Activer OSC |
| `remote.osc.bind` | `127.0.0.1` | Adresse d'écoute |
| `remote.osc.port` | `7381` | Port UDP (1024–65535) |
| `remote.osc.allowed_sources` | cet ordinateur | Adresses ou sous-réseaux dont les paquets sont acceptés |
| `remote.osc.max_subscribers` | `16` | Abonnés simultanés |
| `remote.osc.subscription_ttl_secs` | `60` | Un abonnement non renouvelé dans ce délai prend fin |
| `remote.events.position_interval_ms` | `250` | Fréquence de publication des temps pendant la lecture |

Les valeurs hors plage sont corrigées au chargement du fichier, et la
correction est consignée dans le journal.
