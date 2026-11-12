# Fauste Player — Guia de l'usuari

<div class="ai-notice" role="note">Aquesta guia s'ha traduït amb IA a partir de la guia en anglès (v0.14.0-144-g8517d951, commit 8517d95) i pot contenir errors. En cas de qualsevol diferència, preval la versió en anglès. <a href="../index.html" hreflang="en">Llegeix l'original en anglès</a></div>

Fauste Player és una aplicació d'emissió (playout) per a estudis de ràdio.
Executa diversos **reproductors** independents, un al costat de l'altre. Cada
reproductor té les seves pròpies llistes de reproducció i el seu propi
transport. Els reproductors poden encadenar pistes amb mescles superposades,
preescoltar en una sortida **CUE** a part i sobreviure a les fallades sense
posar res en antena per si sols.

> **Desenvolupat amb ajuda d'IA i oferit tal qual.** Fauste Player, aquesta
> guia i les seves traduccions s'han desenvolupat amb ajuda substancial
> d'eines d'IA. S'ofereix sense cap garantia i l'utilitzes sota el teu propi
> risc: consulta l'[avís legal](https://github.com/sanchezfauste/fauste-player#disclaimer).

![Pantalla principal](../../images/main-screen.png)

| Guia | Què tracta |
|---|---|
| [Primers passos](getting-started.md) | Instal·lació, primer inici, afegir música, reproduir |
| [Reproductors](players.md) | La columna del reproductor: transport, modes, compte enrere, forma d'ona, CUE |
| [Llistes de reproducció](playlists.md) | Pestanyes, la taula de pistes, arrossegar i deixar anar, el menú contextual, fitxers M3U/PLS |
| [Cartutxera](cartwall.md) | Cartutxos instantanis: disparar, pàgines, bucles, cartutxos exclusius, configuració |
| [Biblioteca](library.md) | La finestra de la biblioteca de pistes: categories, tipus, cerca, preescolta, l'editor extern i els fitxers canviats |
| [Disposició](layout.md) | Mostrar i amagar panells, amplades dels reproductors, moure i redimensionar la cartutxera, maximitzar un panell, el menú de disposició |
| [Automatització](automation.md) | La columna Graella, el seu editor, el musicalitzador i les seves rotacions |
| [Marcadors i mescla](markers-and-mixing.md) | Cue-in/cue-out, el punt MIX, intro i outro, anàlisi automàtica |
| [Configuració](settings.md) | Sortides d'àudio, reproductors, anàlisi, llistes, MIDI, control remot |
| [Normalització de la sonoritat](loudness.md) | Totes les pistes i cartutxos amb la mateixa sonoritat mitjançant un guany fix per pista: la insígnia de guany, excloure i ajustar una pista, sortides bit perfect |
| [Sortida bit perfect](bit-perfect.md) | Un mode especial: sense fosos encadenats, volum al 100 %. Dispositius exclusius que segueixen la freqüència de cada fitxer, el distintiu BP, sortida DSD, comprovar una cadena |
| [Teclat](keyboard.md) | Dreceres |
| [Superfícies de control MIDI](midi.md) | Botons, faders i llums en controladors MIDI |
| [Control remot](remote-control.md) | Manejar Fauste Player per la xarxa: API HTTP, seguretat |
| [Dades i còpies de seguretat](data-and-backups.md) | On es desen els fitxers, desament automàtic, recuperació després d'una fallada, mode portàtil |
| [Android](android.md) | L'aplicació en telèfons i tauletes: instal·lació, la disposició tàctil, música al dispositiu |
| [Resolució de problemes](troubleshooting.md) | Sense so, dispositiu perdut, fitxers marcats com a no disponibles, registres |

La interfície està disponible en anglès, castellà, català, neerlandès,
francès, gallec, alemany, italià, basc, polonès i portuguès. Segueix l'idioma
del sistema operatiu tret que en triïs un a Configuració. La interfície
en tots els idiomes, l'anglès i el castellà inclosos, s'ha produït amb IA i
pot contenir errors. Les correccions són benvingudes com a issues o pull
requests.

La guia també està disponible en altres idiomes, traduïda amb IA a partir
de la guia en anglès: a la [guia en línia](https://sanchezfauste.com/fauste-player/guide/),
tria'n un amb el botó del globus de la barra superior.
Una pàgina traduïda ho indica a dalt i enllaça amb la pàgina en
anglès, que és la de referència quan les dues no coincideixen.
