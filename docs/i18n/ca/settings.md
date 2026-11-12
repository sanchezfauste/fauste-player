# Configuració

Obre **Configuració** a la barra superior. Tanca-la amb **Tancar** o amb
`Esc`. La majoria dels canvis s'apliquen a l'instant i es desen
automàticament.

La finestra té una sola mida (900 × 640, més petita en una pantalla petita)
sigui quina sigui la secció, i la secció es desplaça dins seu. Cada secció
alinea les seves etiquetes en una sola columna.

Les seccions Reproductors, Vúmetres, Anàlisi, Sonoritat, Automatització i Dreceres de teclat tenen un
botó **Restaurar els valors per defecte** a la capçalera. Demana confirmació
i després restableix només aquella secció (Reproductors conserva el nombre de
reproductors i l'idioma; Dreceres no té cap altre botó de restabliment).
Sortides d'àudio, Llistes, Cartutxera, Biblioteca, MIDI i Remot no en tenen.

## Ajustos pendents {#settings-pending}

Tots els ajustos s'apliquen mentre l'aplicació s'executa. El sistema d'àudio,
la freqüència de mostreig, la mida del búfer (també la pròpia d'un
dispositiu), les sortides Main i Cue (reproductors i cartutxera), els
dispositius bit perfect i els ajustos DSD canvien com s'obre un dispositiu de
sortida, de manera que s'apliquen tan bon punt és segur:

- Els ajustos d'un dispositiu s'apliquen quan no hi sona res: cap reproductor
  dirigit a ell no està sonant, fent un fos ni preescoltant (CUE, també quan
  és retingut), i cap cartutx hi sona. Un reproductor en pausa, o amb una pista
  només carregada, no reté un dispositiu: la seva pista es queda en pausa on
  era.
- La sortida d'un reproductor o de la cartutxera es mou quan aquell
  reproductor (o la cartutxera) no sona; els altres reproductors als mateixos
  dispositius no s'interrompen.
- Un sistema d'àudio nou s'aplica quan no sona res enlloc.

Un canvi en espera s'executa en el primer moment en què no sona res del que
afecta. Pot ser l'interval entre el final d'una pista i el començament de la
següent: la pista següent llavors comença més tard pel temps que triga el
dispositiu a tornar-se a obrir. **Aplicar ara** (més avall) força el canvi a
l'instant.

Una freqüència o un búfer donats a un dispositiu només compten quan canvien
amb què el dispositiu s'obre: donar a un dispositiu el mateix valor que el
global, o esborrar un valor així, no canvia res.

Fins aleshores la barra superior mostra **Ajustos pendents**. Posa-hi el
ratolí a sobre per veure què espera i per què; fes-hi clic per veure la llista,
que també mostra qualsevol canvi que un dispositiu o una sortida hagi refusat
(el dispositiu conserva els seus ajustos actuals; el canvi es torna a provar
quan el canvies o prems **Aplicar ara**). El peu de Configuració diu «Alguns
canvis esperen que les sortides que afecten quedin lliures.» i ofereix
**Aplicar ara**; un avís breu (per exemple, que s'ha desat un ajust) pot
ocupar un moment el lloc d'aquest text.

**Aplicar ara** aplica a l'instant tots els canvis de sortida en espera.
Quan això interrompria alguna cosa, primer llista els dispositius i el que hi
sona; **Interrompre i aplicar** interromp llavors breument l'àudio d'aquests
dispositius: el que sonava continua d'on era després d'un breu buit, i no
comença res del que era en pausa o aturat. **Cancel·lar** (o `Esc`) continua
esperant.

Els límits i els ajustos del motor s'editen al fitxer de configuració amb
l'aplicació tancada (vegeu [Dades i còpies de seguretat](data-and-backups.md)).

## Sortides d'àudio {#audio-outputs}

Els canvis d'aquesta secció s'apliquen mentre l'aplicació s'executa, tan bon
punt no hi sona res del que afecten: vegeu [Ajustos pendents](#settings-pending).

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
| Mida del búfer | Fotogrames per bloc d'àudio, tret que un dispositiu en tingui una de pròpia; la latència resultant es mostra a sota. Les sortides a través d'un servidor de so o d'un mesclador del sistema (PulseAudio, el dispositiu `default` d'ALSA) demanen almenys 50 ms sigui quina sigui aquesta mida, o tant com accepti el dispositiu, perquè no se senti un ordinador ocupat; la latència llavors ho diu (`shared_buffer_ms` al fitxer de configuració) |
| Sortides per reproductor | Per a cada reproductor, i per a la columna **Música** (el [musicalitzador](automation.md)), un dispositiu **Main** (en antena) i un dispositiu **Cue** (preescolta), cadascun amb un parell de canals. Una targeta de so que ofereix diversos perfils de sortida (ALSA llista frontal, surround, maquinari directe…) mostra cadascun com a *targeta — perfil*; dues entrades que encara es llegirien igual reben l'identificador del dispositiu entre parèntesis. Les interfícies multicanal poden portar diversos reproductors en parells diferents. Es permet una sortida Cue que comparteix una sortida d'antena (el Main de qualsevol reproductor o el de la cartutxera): una nota a sota diu que la seva preescolta se sent a l'antena i que pregunta primer mentre hi sona alguna cosa. |
| Provar Main / Provar Cue | Reprodueix un to breu (1 kHz a Main, 440 Hz a Cue, 1,5 s, −18 dBFS) a la sortida triada, de manera que puguis comprovar el cablejat abans de sortir en antena |
| Biblioteca | La sortida Cue de la biblioteca, amb el seu propi **Provar Cue**: on se sent la [preescolta](library.md#pre-listening) de la finestra de la biblioteca (en un mòbil, la de la pestanya Biblioteca). Sense ella, Preescolta de la barra de previsualització és atenuat. A Android la seva elecció buida diu *Predeterminat del sistema* en lloc de *Cap*: la preescolta i **Provar Cue** sonen llavors per la sortida predeterminada (vegeu [Android → Biblioteca](android.md#library)). Una sortida Biblioteca en una sortida d'antena funciona com el Cue d'un reproductor: una nota a sota diu que la seva preescolta se sent a l'antena i que pregunta primer mentre hi sona alguna cosa. És a les vistes Bàsic i Avançat, també en un mòbil |
| Cartutxera | Les sortides Main i Cue de la cartutxera. Main per defecte és la sortida del sistema. Sense Cue no hi ha preescolta dels cartutxos; un Cue en una sortida d'antena funciona com als reproductors. |
| Freqüència de mostreig: *dispositiu* (Avançat) | **Global (...)** usa la freqüència de mostreig de dalt; un valor dona a aquest dispositiu la seva pròpia freqüència. Només s'ofereixen les freqüències que el dispositiu indica; una freqüència desada que ja no indica continua llistada amb una nota que potser no s'obrirà (el dispositiu torna aleshores a la freqüència global). Els valors propis només s'apliquen als dispositius que nomena una sortida, no a la sortida predeterminada del sistema tret que n'hi hagi una que ho faci. |
| Mida del búfer: *dispositiu* (Avançat) | **Global (...)** usa la mida del búfer de dalt; (amb el mínim que demana una sortida compartida, com més amunt); un valor dona a aquest dispositiu la seva pròpia, usada tal com és, amb la latència a sota. Un dispositiu que no accepta una mida de búfer pròpia torna a la global, i també a la freqüència global quan tampoc no accepta una freqüència pròpia. |
| Bit perfect: *dispositiu* (Avançat) | Desactivat per defecte. Un mode especial per a l'escolta audiòfila, no per a programes mesclats: el dispositiu s'obre amb accés exclusiu i segueix la freqüència de mostreig de cada pista. Els reproductors la sortida Main dels quals hi és mai no fan fos encadenat ni mesclen, un canvi de freqüència deixa un breu silenci entre dues pistes, i el seu volum es manté al 100 %. L'interruptor és desactivat allà on el dispositiu no pot donar accés exclusiu. Vegeu [Què costa](bit-perfect.md#what-it-costs). |
| DSD: *dispositiu* (Avançat) | **Convertir a PCM** (per defecte), **DoP** o, a Linux, **DSD natiu**. Tots els dispositius ho mostren; només s'ofereixen els modes que el dispositiu pot acceptar, i una línia a sota diu per què no els altres. Vegeu [DSD](bit-perfect.md#dsd). |
| Quan una altra font necessita una sortida DSD (Avançat) | **Continuar la pista DSD en PCM** (per defecte), o **Mantenir el DSD i silenciar les altres fonts**. Vegeu [DSD](bit-perfect.md#dsd). |
| Silenci DSD (Avançat) | Silenci enviat abans que comenci un flux DSD, quan acaba i en passar a PCM, perquè el convertidor s'enganxi sense espetec; 200 ms per defecte, de 0 a 2000. |

Una sortida Cue mai no torna a la sortida que usa Main. Un Cue que anomena un
dispositiu d'un sistema d'àudio que aquest ordinador no té vol dir «sense
cue». Es permet un Cue en una sortida d'antena (el mateix dispositiu i canals
que un Main, o que se superposin): una nota a sota diu que la seva preescolta
se sent a l'antena i que pregunta primer mentre hi sona alguna cosa. Un
reproductor sense sortida Cue té el botó **CUE** atenuat; en posar-hi el
ratolí a sobre et diu que triïs aquí una sortida Cue. El mateix val per a
**Preescoltar al CUE** de la cartutxera.

Si un dispositiu desapareix mentre sona, els reproductors conserven les seves
línies de temps, i el dispositiu es torna a obrir quan torna (vegeu
[Resolució de problemes](troubleshooting.md)).

## Reproductors {#players}

![Configuració, Reproductors: nombre de reproductors, mode per defecte, durada del fos entre pistes, durada de l'stop amb fos, mescla automàtica, cue-in i cue-out, avís de final de pista i idioma](../../images/guide/settings-players.png)

| Ajust | Per defecte | Significat |
|---|---|---|
| Idioma | Sistema | Idioma de la interfície |
| Nombre de reproductors | 4 | Columnes de la pantalla principal (no es pot treure un reproductor en antena). El nombre s'aplica quan deixes anar el control lliscant; un nombre escrit al quadre s'aplica quan prems Enter o surts del quadre, i Escape el cancel·la |
| Mode per defecte | CONT | El mode amb què comencen els reproductors |
| Durada del fos entre pistes | 1000 ms | Quant triga a fer el fos de sortida la pista en antena quan prems Play (següent) mentre sona, i per a Pista anterior. No s'aplica a un reproductor la sortida Main del qual és bit perfect |
| Durada de l'Stop amb fos | 2000 ms | Quant triga **Stop amb fos** a fer el fos de sortida de la pista. S'aplica a tots els reproductors, també als bit perfect |
| Mescla automàtica al punt MIX | Activada | Superposa les pistes en mode continu. No s'aplica a un reproductor la sortida Main del qual és bit perfect |
| Usar cue-in i cue-out | Activat | Desactivat: els reproductors reprodueixen cada pista de l'inici al final del fitxer; els marcadors es conserven i els cartutxos continuen usant els seus. Les durades i els totals de les llistes segueixen el mateix interval |
| Avís de final de pista | 10 s | Quan el compte enrere comença a parpellejar en vermell |

Mentre la sortida Main d'un reproductor és bit perfect, una nota sota **Mescla
automàtica al punt MIX** diu que aquests reproductors mai no mesclen ni fan fos
encadenat, de manera que la durada del fos entre pistes i la mescla
automàtica no s'hi apliquen. Tots dos ajustos continuen editables: encara s'apliquen als altres
reproductors. Vegeu [Què costa](bit-perfect.md#what-it-costs).

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

Els llindars descrits a [Marcadors i mescla](markers-and-mixing.md); cada fila
té sota el seu nom una indicació d'una línia que diu què fa el valor.
**Tornar a analitzar totes les pistes** executa de nou l'anàlisi per a tota
la biblioteca, llegint cada fitxer de nou; els marcadors manuals es conserven.
Mentre s'executa, el botó és atenuat i mostra fins on ha arribat
(**Analitzant… 120 de 340**), de manera que prémer-lo de nou no comença de nou;
la informació emergent de **Analitzant N pistes** de la barra d'estat dona el
mateix progrés. Canviar-hi un llindar també analitza de nou la biblioteca, i
reinicia una execució ja en marxa perquè cap pista conservi marcadors dels
valors antics.

Després d'una actualització en què ha canviat l'anàlisi, les pistes
analitzades per la versió anterior conserven els marcadors i les formes
d'ona, que continuen funcionant. A l'inici, Fauste Player diu quantes n'hi ha
i ofereix **Analitzar ara** o **Més tard**; **Analitzar les pistes
desactualitzades (N)** d'aquí fa el mateix en qualsevol moment. Les pistes
dels reproductors s'actualitzen igualment, a mesura que es mostren, i també
les pistes dels cartutxos que no tenen format registrat (un cartutx només es
reprodueix bit perfect quan se'n coneix el format). Les pistes el fitxer de
les quals falta no es compten fins que el fitxer torna.

L'anàlisi també mesura la sonoritat de cada fitxer per a la [normalització de
la sonoritat](loudness.md). Els fitxers analitzats abans que es mesurés la
sonoritat es mesuren en segon pla, quan l'anàlisi no té res més a fer, tant si
la normalització és activada com desactivada; els seus marcadors no es toquen.
**Tornar a analitzar** (al menú de la llista) i **Tornar a analitzar totes les
pistes** tornen a mesurar.

**Cercar fitxers canviats** revisa totes les pistes de la biblioteca a la
recerca d'un fitxer que un altre programa hagi canviat mentre Fauste Player
s'executava, i torna a analitzar els canviats; un avís diu quants n'ha trobat.
El botó apareix atenuat mentre una cerca és en marxa. Vegeu [Quan un fitxer
canvia](library.md#when-a-file-changes).

## Sonoritat {#loudness}

![Configuració, Sonoritat: normalització activada, amb l'objectiu, el sostre de true peak, el guany màxim i l'ajust màxim als seus valors per defecte](../../images/guide/settings-loudness.png)

Reprodueix cada pista i cada cartutx amb la mateixa sonoritat, amb un guany fix
per pista. La [normalització de la sonoritat](loudness.md) explica com es
calcula el guany, el distintiu de la capçalera del reproductor i els ajustos
per pista.

| Ajust | Per defecte | Interval | Significat |
|---|---|---|---|
| Normalitzar la sonoritat | Activada | | Normalització activada o desactivada |
| Objectiu | −16 LUFS | −36 … −6 LUFS, passos de 0,5 | La sonoritat integrada a la qual es porta cada pista |
| Sostre de true peak | −1 dBTP | −12 … 0 dBTP, passos de 0,1 | Una pujada mai porta el true peak de la pista per sobre d'això. Les baixades no es limiten |
| Guany màxim | 12 dB | 0 … 24 dB | El guany més gran que pot rebre una pista fluixa. Les baixades no es limiten |
| Ajust màxim | 12 dB | 0 … 24 dB | Fins on pot arribar l'ajust manual de cada pista, amunt o avall |

Els canvis s'apliquen a les pistes que es carreguin després: el que sona
conserva el seu guany, i la pista següent que espera a cada reproductor es torna
a preparar amb el nou. Una nota sota els ajustos diu quantes pistes encara
esperen la seva mesura; sonen sense canvis fins que s'acabi. Un reproductor o un
cartutx en una sortida bit perfect no es normalitza mai. **Restaurar els valors
per defecte** restableix només aquesta secció (normalització activada). La
normalització és activada per a les instal·lacions noves; una configuració
existent conserva el seu ajust actual (cada configuració desada ja en guarda el
valor).

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

La interfície en tots els idiomes, l'anglès i el castellà inclosos, s'ha
produït amb IA i pot contenir errors. **Quant a Fauste Player** porta un avís
legal general que els cobreix tots. Les correccions són benvingudes com a
issues o pull requests.

## Cartutxera {#cartwall}

![Configuració, Cartutxera: les pàgines, la graella i l'editor del cartutx seleccionat](../../images/guide/settings-cartwall.png)

Pàgines, mida de la graella, l'editor de cartutxos, i importació i exportació
de pàgines de cartutxos. Vegeu [Cartutxera](cartwall.md).

## Biblioteca {#library}

Crea, canvia el nom, el color i el tipus, mou (arrossegant) i elimina les
categories que agrupen les teves pistes, vigila una carpeta amb una categoria i
treu-ne les pistes que falten, sense obrir la finestra de la biblioteca. Vegeu
[Biblioteca](library.md#categories) i
[Carpetes vigilades](library.md#watched-folders).

**Editor extern** és l'editor d'àudio que *Obrir a l'editor extern* inicia amb
el fitxer d'una pista: la ruta del programa (o *Cap*), **Triar…** per
escollir-lo i **Treure** per eliminar-lo. A macOS es pot triar un paquet
d'aplicació (`.app`). Si el programa no es pot iniciar, un avís ho diu. La fila
no es mostra a Android, ni a les compilacions Flatpak i snap. Vegeu [Quan un
fitxer canvia](library.md#when-a-file-changes).

## Automatització {#automation}

Les rotacions i les regles del [musicalitzador](automation.md) (no a Android).

![Configuració, Automatització: les rotacions Música (la per defecte) i Hits, la Música seleccionada amb el seu únic pas sobre tota la música, i les regles del musicalitzador](../../images/guide/settings-automation.png)

- **Rotacions:** la llista, amb la per defecte marcada. **Nova rotació** n'afegeix
  una que reprodueix tota la música, **Fes-la per defecte** converteix la
  seleccionada en la per defecte, **Esborra la rotació** l'elimina (l'última no
  es pot esborrar).
- **La rotació seleccionada:** el seu **Nom**, i després una fila per **pas**:
  d'on surten les seves pistes (una categoria amb les seves subcategories, o
  *Tot* d'un tipus), quantes en reprodueix cada vegada (**Com a mínim** 0–99,
  **Com a màxim** 1–99; amb mínim 0 el pas es salta alguns torns), **Puja**,
  **Baixa** i **Treu**. **Afegeix un pas** n'afegeix un al final. Un pas la
  categoria del qual s'ha esborrat ho diu; el musicalitzador se'l salta.
- **Musicalitzador:** quant temps ha de passar abans que la mateixa pista pugui
  tornar a sonar (3 hores, o quan hagi sonat la meitat de les altres pistes del
  seu pas, el que passi primer; 0 desactiva qualsevol dels dos), el mateix
  artista (30 minuts), el **marge per endavant** (30 minuts de música en cua) i
  les **pistes sonades que es conserven** a la seva llista (50).

**Restaurar els valors per defecte** restableix les regles; les rotacions es queden.

## Dreceres de teclat {#keyboard-shortcuts}

![Configuració, Dreceres de teclat: cada acció dels reproductors amb la seva tecla, i Treure al costat de les assignades](../../images/guide/settings-shortcuts.png)

Vegeu [Teclat](keyboard.md).

## MIDI {#midi}

![Configuració, MIDI, amb el control MIDI desactivat](../../images/guide/settings-midi.png)

Activa les superfícies de control MIDI, mira els ports d'entrada (**Reconnecta
els ports** els torna a obrir, com si tornessis a connectar els controladors) i
aprèn un control per a cada acció dels reproductors. Vegeu
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
