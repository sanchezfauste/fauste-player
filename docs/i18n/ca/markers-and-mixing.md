# Marcadors i mescla

Cada pista té fins a cinc **marcadors**, en segons:

| Marcador | Significat | Com es defineix |
|---|---|---|
| Cue-in | On comença la reproducció | Automàtic: just abans del primer so per sobre del llindar de retall |
| Cue-out | On acaba la pista | Automàtic: just després de l'últim so per sobre del llindar de retall |
| MIX (inici del segue) | On comença la pista següent en mode continu | Automàtic (vegeu més avall) |
| Inici de l'outro | On comença el final de la pista | Automàtic (vegeu més avall) |
| Final de la intro | Final de la introducció parlada per sobre | A mà, o a partir d'una etiqueta `INTRO` del fitxer |

Els marcadors definits a mà sempre tenen prioritat: una anàlisi nova mai no
els substitueix.

## Editar marcadors {#editing-markers}

A la forma d'ona d'un reproductor, o a la de la seva finestra de CUE (el
mateix menú i els mateixos nanses; un canvi es veu a tots dos alhora):

- **Clic dret** allà on vulguis un marcador i tria **Posar el cue-in aquí**,
  **Posar el final de la intro aquí**, **Posar l'inici de l'outro aquí**,
  **Posar el punt MIX aquí** o **Posar el cue-out aquí**. **Tornar als
  marcadors automàtics** treu els marcadors que has col·locat, i la pista
  s'analitza de nou.
- **Mantén Alt** (Option a macOS): apareixen nanses als marcadors. Arrossega'n
  una per moure'l; el temps es mostra mentre arrossegues. Un arrossegament
  mai no mou el cap de reproducció.

El cue-in ha de quedar abans del cue-out. Els altres marcadors es mantenen
entre tots dos. Els canvis a la pista en antena s'apliquen de seguida a la
seva propera transició.

## L'etiqueta INTRO {#the-intro-tag}

Un fitxer pot portar el temps de la seva intro en una etiqueta `INTRO`, en
segons (`12.5`) o com a `m:ss`. El nom es pot escriure amb qualsevol
combinació de majúscules i minúscules (`INTRO`, `Intro`). Pot ser un marc de
text d'usuari ID3v2 (MP3, WAV, AIFF, DSF), un comentari Vorbis, Opus o FLAC,
un element APE (WavPack, Monkey's Audio) o un àtom lliure d'MP4. Es llegeix
durant l'anàlisi. Un final de la intro manual continua tenint prioritat.

## Com es troben els marcadors automàtics {#how-the-automatic-markers-are-found}

L'anàlisi mesura els pics de la pista en passos de 10 ms i la seva sonoritat
en finestres curtes (50 ms per defecte).

- **Cue-in / cue-out:** només s'omet el quasi-silenci de l'inici i del final:
  tot allò el pic del qual arriba al *llindar de retall* (−60 dBFS per
  defecte), en qualsevol dels dos canals, es conserva, amb un *marge de
  retall* (20 ms per defecte) al voltant. Els fos d'entrada suaus, les cues
  fluixes i els sons curts mai no es tallen.
- **MIX:** l'anàlisi troba l'últim punt on la pista encara està menys de la
  *caiguda per al segue* (15 dB per defecte) per sota de la seva pròpia
  sonoritat típica, de manera que els màsters forts i fluixos amb el mateix
  fos es mesclen de la mateixa manera. Aquest punt mai no és a més de la
  *durada màxima de la mescla* (4 s per defecte) abans del cue-out, de manera
  que les superposicions es mantenen curtes.
- **Outro:** l'anàlisi recorre la pista cap enrere des del cue-out i troba on
  el nivell baixa més de la *caiguda de nivell de l'outro* (6 dB per defecte)
  per sota de la sonoritat mitjana de la pista. L'outro mai no és més llarg
  de 30 s per defecte.
- Les pistes més curtes que la *durada mínima per als marcadors de mescla i
  outro* (60 s per defecte), com ara jingles i anuncis, no tenen MIX ni outro.

### Gravacions llargues {#long-recordings}

Un programa sencer (d'una, quatre o més hores) s'analitza com una cançó,
mentre es reprodueix si cal: un fitxer FLAC o Opus de 4 hores triga menys
d'un minut en un ordinador actual, i la memòria no creix amb la durada.
Cercar qualsevol punt, fins i tot prop del final, és immediat. La forma d'ona
i els marcadors es conserven a la memòria cau d'anàlisi fins a unes 16 hores
d'àudio; un fitxer més llarg també funciona, però s'analitza de nou cada cop
que s'inicia Fauste Player.

Tots aquests valors són a **Configuració → Anàlisi**. Després de canviar-los,
les pistes s'analitzen de nou automàticament.

## Què en fa el reproductor {#what-the-player-does-with-them}

- **Mode continu amb la mescla automàtica activada:** al punt MIX la pista
  següent comença a nivell ple mentre l'actual fa un fos de sortida fins al
  seu cue-out. La superposició és exacta a la mostra.
- **Mode continu sense punt MIX,** o amb la mescla automàtica desactivada: la
  pista següent comença exactament al cue-out, sense pausa.
- **Mode single**, o **Stop al final**: el reproductor s'atura al cue-out.
- **Prémer Play en antena:** la pista següent comença a l'instant i l'actual
  fa un fos de sortida durant la *durada del fos* (1 s per defecte).
- **Usar cue-in i cue-out desactivat** (Configuració → Reproductors): cada
  reproductor reprodueix cada pista de 0 al final del fitxer. El cue-in i el
  cue-out, automàtics i manuals, es conserven, i la forma d'ona els dibuixa
  com a línies tènues. El punt MIX, la intro i l'outro continuen funcionant,
  dins de tot el fitxer; **Mescla automàtica al punt MIX** és un interruptor
  a part. Els comptes enrere, la columna de durada, els totals de les llistes
  a Configuració i els temps de l'API remota segueixen el mateix interval. Els
  cartutxos sempre usen el seu propi cue-in i cue-out. Canviar l'ajust mai no
  reinicia, cerca ni atura una pista que està sonant; la pista següent es
  prepara de nou.

Una pista es pot reproduir abans que acabi la seva anàlisi. Fins aleshores es
reprodueix de l'inici al final del fitxer, sense punt MIX.
