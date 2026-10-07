# Configuració

Obre **Configuració** a la barra superior. Tanca-la amb **Tancar** o amb
`Esc`. La majoria dels canvis s'apliquen a l'instant i es desen
automàticament.

La finestra té una sola mida (900 × 640, més petita en una pantalla petita)
sigui quina sigui la secció, i la secció es desplaça dins seu. Cada secció
alinea les seves etiquetes en una sola columna.

Les seccions Reproductors, Vúmetres, Anàlisi i Dreceres de teclat tenen un
botó **Restaurar els valors per defecte** a la capçalera. Demana confirmació
i després restableix només aquella secció (Reproductors conserva el nombre de
reproductors i l'idioma; Dreceres no té cap altre botó de restabliment).
Sortides d'àudio, Llistes, Cartutxera, MIDI i Remot no en tenen.

## Reinici pendent {#restart-pending}

Alguns canvis només tenen efecte quan l'aplicació torna a iniciar-se: el
sistema d'àudio, la freqüència de mostreig, la mida del búfer (també la
pròpia d'un dispositiu), les sortides Main i Cue (reproductors i cartutxera),
els dispositius bit perfect i els ajustos DSD. El nombre de reproductors no
n'és un: s'aplica a l'instant.

Una freqüència o un búfer donats a un dispositiu només compten quan canvien
amb què el dispositiu s'obre: donar a un dispositiu el mateix valor que el
global, o esborrar un valor així, no queda pendent.

Els límits i els ajustos del motor també s'apliquen a l'inici següent, però
s'editen al fitxer de configuració amb l'aplicació tancada (vegeu
[Dades i còpies de seguretat](data-and-backups.md)), de manera que mai no
apareixen com a pendents.

Mentre un d'aquests està en espera, el peu de Configuració diu «Alguns canvis
s'apliquen després de reiniciar.» i ofereix **Reiniciar ara**, i la barra
superior mostra una píndola **Reinici pendent**. Posa-hi el ratolí a sobre
per veure què està esperant. Un avís breu (per exemple, que s'ha desat un
ajust) pot ocupar un moment el lloc del text del peu; **Reiniciar ara** es
queda. Tots dos fan el mateix:

- Quan no hi ha res en antena, **Reiniciar ara** (o la píndola) reinicia a
  l'instant.
- Quan hi ha alguna cosa en antena, apareix la finestra que llista el que
  sona, amb **Aturar i reiniciar** o **Cancel·lar**.

Primer es desa la sessió i s'aturen l'àudio i el control MIDI, després
l'aplicació torna a iniciar-se amb la mateixa carpeta de dades
(`FAUSTE_HOME`), i després no es posa res en antena per si sol. Si
l'aplicació no es pot tornar a iniciar (en un Flatpak, també quan la nova no
s'inicia a temps), ho diu; inicia-la des del menú d'aplicacions.

## Sortides d'àudio {#audio-outputs}

Els canvis d'aquesta secció esperen un reinici: vegeu
[Reinici pendent](#restart-pending).

![Configuració, Sortides d'àudio, vista Bàsic: el selector, el sistema d'àudio, la freqüència de mostreig, la mida del búfer i les sortides Main i Cue de cada reproductor (aquí el sistema silenciós)](../../images/guide/settings-outputs.png)

A dalt, **Mostrar** tria entre **Bàsic** i **Avançat**. Bàsic mostra el
sistema d'àudio, la freqüència de mostreig, la mida del búfer i les sortides.
Avançat afegeix, per a cada dispositiu que utilitza una sortida, la seva
pròpia freqüència i búfer, l'interruptor bit perfect i el mode DSD, i després
els ajustos DSD. Canviar de vista només mostra o amaga files: no es canvia ni
es restableix res. Quan Bàsic amaga un ajust que s'està usant, una línia ho
diu. Quan cap sortida no utilitza ja un dispositiu, la seva pròpia
freqüència i búfer, el seu interruptor bit perfect i el seu mode DSD
s'obliden la propera vegada que s'inicia l'aplicació: si després una sortida
el torna a usar, comença amb els valors globals. Fins aleshores, triar-lo de
nou (per exemple després d'intercanviar dos dispositius) els conserva.

| Ajust | Significat |
|---|---|
| Sistema d'àudio | L'última opció, **Sense sortida (silenci)**, no reprodueix res: les línies de temps avancen al ritme del temps real sense targeta de so (per a una màquina que no en té, o per assajar). Linux: PipeWire (a les compilacions que l'inclouen), PulseAudio, JACK o ALSA. Windows: WASAPI, ASIO (a les compilacions que l'inclouen) o JACK. macOS: Core Audio o JACK. Els sistemes que falten en aquest ordinador, o sense dispositiu de sortida (un servidor JACK que no s'està executant), es mostren com a no disponibles. «Predeterminat del sistema» usa el primer disponible en aquest ordre. |
| Freqüència de mostreig | La freqüència a la qual funciona cada sortida tret que un dispositiu en tingui una de pròpia (Avançat); els fitxers s'hi converteixen amb un remostreig d'alta qualitat. Els dispositius bit perfect comencen a la seva freqüència i després segueixen els fitxers. |
| Mida del búfer | Fotogrames per bloc d'àudio, tret que un dispositiu en tingui una de pròpia; la latència resultant es mostra a sota |
| Sortides per reproductor | Per a cada reproductor, un dispositiu **Main** (en antena) i un dispositiu **Cue** (preescolta), cadascun amb un parell de canals. Una targeta de so que ofereix diversos perfils de sortida (ALSA llista frontal, surround, maquinari directe…) mostra cadascun com a *targeta — perfil*; dues entrades que encara es llegirien igual reben l'identificador del dispositiu entre parèntesis. Les interfícies multicanal poden portar diversos reproductors en parells diferents. |
| Provar Main / Provar Cue | Reprodueix un to breu (1 kHz a Main, 440 Hz a Cue, 1,5 s, −18 dBFS) a la sortida triada, de manera que puguis comprovar el cablejat abans de sortir en antena |
| Cartutxera | Les sortides Main i Cue de la cartutxera. Main per defecte és la sortida del sistema. Sense Cue no hi ha preescolta dels cartutxos. |
| Freqüència de mostreig: *dispositiu* (Avançat) | **Global (...)** usa la freqüència de mostreig de dalt; un valor dona a aquest dispositiu la seva pròpia freqüència. Només s'ofereixen les freqüències que el dispositiu indica; una freqüència desada que ja no indica continua llistada amb una nota que potser no s'obrirà (el dispositiu torna aleshores a la freqüència global). Els valors propis només s'apliquen als dispositius que nomena una sortida, no a la sortida predeterminada del sistema tret que n'hi hagi una que ho faci. |
| Mida del búfer: *dispositiu* (Avançat) | **Global (...)** usa la mida del búfer de dalt; un valor dona a aquest dispositiu la seva pròpia, amb la latència a sota. Un dispositiu que no accepta una mida de búfer pròpia torna a la global, i també a la freqüència global quan tampoc no accepta una freqüència pròpia. |
| Bit perfect: *dispositiu* (Avançat) | Un dispositiu bit perfect s'obre amb accés exclusiu i segueix la freqüència de mostreig de cada fitxer mentre no hi sona res. L'interruptor és desactivat allà on el dispositiu no pot donar accés exclusiu. Vegeu [Sortida bit perfect](bit-perfect.md). |
| DSD: *dispositiu* (Avançat) | **Convertir a PCM** (per defecte), **DoP** o, a Linux, **DSD natiu**. Tots els dispositius ho mostren; només s'ofereixen els modes que el dispositiu pot acceptar, i una línia a sota diu per què no els altres. Vegeu [DSD](bit-perfect.md#dsd). |
| Quan una altra font necessita una sortida DSD (Avançat) | **Continuar la pista DSD en PCM** (per defecte), o **Mantenir el DSD i silenciar les altres fonts**. Vegeu [DSD](bit-perfect.md#dsd). |
| Silenci DSD (Avançat) | Silenci enviat abans que comenci un flux DSD, quan acaba i en passar a PCM, perquè el convertidor s'enganxi sense espetec; 200 ms per defecte, de 0 a 2000. |

Una sortida Cue mai no torna a la sortida que usa Main, de manera que la
preescolta mai no surt en antena. Un Cue que anomena un dispositiu d'un
sistema d'àudio que aquest ordinador no té, o la mateixa sortida (dispositiu
i canals) que Main, vol dir «sense cue». Quan una sortida Cue és la mateixa
que la seva sortida Main, un avís a sota ho diu. Un reproductor sense sortida
Cue, o amb el Cue a la seva sortida Main, té el botó **CUE** atenuat; en
posar-hi el ratolí a sobre et diu que triïs aquí una sortida Cue. El mateix
val per a **Preescoltar al CUE** de la cartutxera.

Si un dispositiu desapareix mentre sona, els reproductors conserven les seves
línies de temps, i el dispositiu es torna a obrir quan torna (vegeu
[Resolució de problemes](troubleshooting.md)).

## Reproductors {#players}

![Configuració, Reproductors: nombre de reproductors, mode per defecte, durada del fos, mescla automàtica, cue-in i cue-out, avís de final de pista i idioma](../../images/guide/settings-players.png)

| Ajust | Per defecte | Significat |
|---|---|---|
| Idioma | Sistema | Idioma de la interfície |
| Nombre de reproductors | 4 | Columnes de la pantalla principal (no es pot treure un reproductor en antena) |
| Mode per defecte | CONT | El mode amb què comencen els reproductors |
| Durada del fos | 1000 ms | Usat per Play en antena i per Stop amb fos |
| Mescla automàtica al punt MIX | Activada | Superposa les pistes en mode continu |
| Usar cue-in i cue-out | Activat | Desactivat: els reproductors reprodueixen cada pista de l'inici al final del fitxer; els marcadors es conserven i els cartutxos continuen usant els seus. Les durades i els totals de les llistes segueixen el mateix interval |
| Avís de final de pista | 10 s | Quan el compte enrere comença a parpellejar en vermell |

## Vúmetres {#meters}

![Configuració, Vúmetres, amb el mesurador de pic digital triat](../../images/guide/settings-meters.png)

Els canvis s'apliquen a l'instant. Configuració només mostra el que usa el
tipus de mesurador triat: els mesuradors EBU, DIN i VU tenen l'escala, la
zona vermella i el comportament que fixa la seva norma (només es defineix el
nivell d'alineació), i l'alineació d'un mesurador K-System és el seu propi 0.
Un valor que has definit es conserva per a quan tornis a triar aquell tipus.

| Ajust | Per defecte | Significat |
|---|---|---|
| Tipus de mesurador | Pic digital | Com puja i baixa la barra, i la seva escala, segons una norma (vegeu més avall) |
| Temps de pujada, Velocitat de caiguda | 5 ms, 11,8 dB/s | Només per a **Personalitzat**. El temps de pujada és un temps d'integració: una ràfega de to tan llarga marca 2 dB per sota; 0 mostra tots els pics. |
| True peak | Desactivat | Només pic digital, personalitzat i K-System. Mesura entre mostres, amb el filtre de sobremostreig 4× que publica l'ITU-R BS.1770. Mostra els pics que superen 0 dBFS després de la conversió, que un mesurador de pic de mostra no veu. Com permet la norma, un clic aïllat d'una sola mostra pot marcar fins a uns 0,3 dB per sota del seu valor de mostra. |
| Mínim de l'escala | −60 dBFS | El fons de l'escala digital (pic digital i personalitzat). Els altres mesuradors mostren l'interval que dona la seva norma. |
| Retenció del pic | 2 s | Només pic digital, personalitzat i K-System: quant de temps es manté il·luminat el nivell més alt; 0 la desactiva. Els mesuradors de programa i el VU no tenen retenció. |
| Nivell d'alineació | −18 dBFS | Tots excepte el K-System. Marcat a l'escala (EBU R68). També és on són la marca EBU TEST, la marca DIN −9 i el 0 VU. |
| Avís des de | −9 dBFS | Groc a partir d'aquí (màxim permès EBU), per als mesuradors de pic digital i personalitzat |
| Perill des de | −3 dBFS | Vermell a partir d'aquí, per als mesuradors de pic digital i personalitzat. Els altres es posen vermells allà on ho fa la seva escala: el VU a partir de 0 VU, els PPM EBU i DIN a partir del màxim permès (EBU +9, DIN 0), el K-System a partir de +4. |
| Lectura de sonoritat | Curt termini | La sonoritat sota el mesurador: desactivada, momentània (últims 400 ms) o a curt termini (últims 3 s), EBU R128 |
| Objectiu de sonoritat | −23 LUFS | La lectura es veu en verd a ±1 LU (EBU R128) |

| Tipus de mesurador | Norma | Comportament |
|---|---|---|
| Pic digital | IEC 60268-18 | Mostra tots els pics a l'instant; cau 20 dB en 1,7 s |
| PPM EBU | IEC 60268-10 tipus IIb | Els pics més curts d'uns 10 ms marquen més baix (una ràfega de to de 10 ms marca uns 1,6 dB per sota, una de 0,5 ms uns 18 dB per sota), dins de les toleràncies d'EBU Tech 3205; cau 24 dB en 2,8 s |
| PPM DIN | IEC 60268-10 tipus I | El mateix amb un temps d'integració de 5 ms; cau 20 dB en 1,5 s |
| VU | IEC 60268-17 | El nivell mitjà, amb el moviment d'agulla d'un VU: 99 % en 300 ms, amb un lleuger sobrepas; una sinusoide marca el seu nivell de pic |
| K-20, K-14, K-12 | K-System | Dues seccions: la mitjana (RMS, 600 ms) com a barra massissa i el pic (cau 26 dB en 3 s) més fosc a sobre. 0 és 20, 14 o 12 dB per sota del fons d'escala; verd per sota de 0, ambre de 0 a +4, vermell per sobre. K-12 convé a la ràdio, K-14 i K-20 a programes més dinàmics. |
| Personalitzat | — | El teu temps de pujada i la teva velocitat de caiguda |

Cada mesurador usa l'escala de la seva norma, amb les marques entre els
canals:

| Mesurador | Escala |
|---|---|
| Pic digital, personalitzat | −60 … 0 dBFS, marques cada 10 dB fins a −40 i cada 5 dB per sobre; els 20 dB superiors ocupen la meitat de l'alçada |
| PPM EBU | −12 … +12 al voltant del nivell d'alineació (TEST), cada 4 dB; els nivells més fluixos descansen al fons |
| PPM DIN | −50 … +5, on 0 és 9 dB per sobre del nivell d'alineació (−9 dBFS per defecte) |
| VU | −20 … +3 VU, 0 VU al nivell d'alineació; la barra es mou en proporció a la tensió, com l'agulla |
| K-System | des de +20, +14 o +12 (0 dBFS) fins a −60; uniforme en dB fins a −24 |

## Anàlisi {#analysis}

![Configuració, Anàlisi: els llindars dels marcadors automàtics](../../images/guide/settings-analysis.png)

Els llindars descrits a [Marcadors i mescla](markers-and-mixing.md).
**Tornar a analitzar totes les pistes** executa de nou l'anàlisi per a tota
la biblioteca; els marcadors manuals es conserven.

Després d'una actualització en què ha canviat l'anàlisi, les pistes
analitzades per la versió anterior conserven els marcadors i les formes
d'ona, que continuen funcionant. A l'inici, Fauste Player diu quantes n'hi ha
i ofereix **Analitzar ara** o **Més tard**; **Analitzar les pistes
desactualitzades (N)** d'aquí fa el mateix en qualsevol moment. Les pistes
dels reproductors s'actualitzen igualment, a mesura que es mostren, i també
les pistes dels cartutxos que no tenen format registrat (un cartutx només es
reprodueix bit perfect quan se'n coneix el format). Les pistes el fitxer de
les quals falta no es compten fins que el fitxer torna.

## Llistes {#playlists}

![Configuració, Llistes: la carpeta de música, les llistes i les columnes de la taula](../../images/guide/settings-playlists.png)

- **Carpeta de música:** on comencen els diàlegs de fitxers.
- **Llista nova**, **canviar el nom** (edita el nom i prem Enter; Esc
  cancel·la) i **eliminar** (icona de paperera).
- **Importar M3U / PLS…** crea una llista nova a partir d'un fitxer de llista.
  **M3U** a cada fila l'exporta com a M3U8. Vegeu
  [Llistes de reproducció](playlists.md).
- **Columnes de la taula:** quines columnes mostren les taules de pistes i en
  quin ordre, per a tots els reproductors: una casella per columna (Títol i
  Dur. no es poden desactivar), fletxes amunt i avall per a les mostrades, i
  **Columnes per defecte**. Vegeu [Llistes de reproducció](playlists.md).

**Idioma:** una llista desplegable: **Sistema** (segueix el sistema
operatiu), i després tots els idiomes en què està disponible la interfície,
cadascun amb el seu propi nom (l'anglès primer, després per ordre alfabètic:
per exemple Español). La interfície canvia a l'instant. Un idioma del sistema
sense traducció pròpia usa el més proper (el francès del Canadà usa el
francès, el portuguès del Brasil usa el portuguès), i l'anglès en cas
contrari. Un idioma del fitxer de configuració que la interfície no té
apareix com a **Sistema** i segueix el sistema operatiu.

L'anglès i el castellà estan escrits a mà. Les altres traduccions s'han
generat amb IA i poden contenir errors; quan se n'usa una, **Quant a Fauste
Player** ho diu. Les correccions de parlants nadius són benvingudes com a
issues o pull requests.

## Cartutxera {#cartwall}

![Configuració, Cartutxera: les pàgines, la graella i l'editor del cartutx seleccionat](../../images/guide/settings-cartwall.png)

Pàgines, mida de la graella, l'editor de cartutxos, i importació i exportació
de pàgines de cartutxos. Vegeu [Cartutxera](cartwall.md).

## Dreceres de teclat {#keyboard-shortcuts}

![Configuració, Dreceres de teclat: cada acció dels reproductors amb la seva tecla, i Treure al costat de les assignades](../../images/guide/settings-shortcuts.png)

Vegeu [Teclat](keyboard.md).

## MIDI {#midi}

![Configuració, MIDI, amb el control MIDI desactivat](../../images/guide/settings-midi.png)

Activa les superfícies de control MIDI, mira els ports d'entrada i aprèn un
control per a cada acció dels reproductors. Vegeu
[Superfícies de control MIDI](midi.md).

## Remot {#remote}

![Configuració, Remot, amb l'API HTTP escoltant en aquest ordinador](../../images/guide/settings-remote.png)

Control remot per la xarxa, per a pàgines web, aplicacions de mòbil,
automatització i superfícies de control. Vegeu
[Control remot](remote-control.md).

- **Permetre el control remot per HTTP**, la seva **adreça** i el seu
  **port**, i una línia que diu si està escoltant.
- **Testimoni**, necessari fora d'aquest ordinador. **Generar** en crea un
  d'aleatori, **Mostrar** el revela i **Copiar** el posa al porta-retalls.
  Apareix un avís quan l'adreça arriba més enllà d'aquest ordinador i no hi
  ha testimoni.
- **Pàgines web que poden usar l'API**: un origen per línia.
- **Permetre el control per OSC**, la seva **adreça** i el seu **port**, i els
  **emissors permesos** (adreces o subxarxes, una per línia).
- **Publicar els temps cada**: amb quina freqüència s'envien els temps
  transcorreguts i restants mentre sona alguna cosa.

Els camps de text i els nombres s'apliquen quan els deixes, cosa que inclou
obrir una altra secció o tancar Configuració; Esc cancel·la el que estaves
escrivint. Un valor no vàlid es corregeix, i el camp mostra el que s'ha
conservat.
