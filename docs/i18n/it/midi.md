# Superfici di controllo MIDI

Fauste Player si può suonare da controller MIDI: controller con pad e fader,
superfici in stile DJ o tastiere. I pulsanti di trasporto e il fader del
volume di ogni lettore si possono associare a un pulsante, a un tasto o a un
fader, e i pulsanti con luci mostrano cosa sta facendo ogni lettore.

## Attivarlo {#turning-it-on}

![Impostazioni, MIDI: l'interruttore per attivare il MIDI, e l'elenco delle azioni con un pulsante Apprendi ciascuna](../../images/guide/settings-midi.png)

Apri **Impostazioni → MIDI** e spunta **Usa superfici di controllo MIDI**.
L'elenco sotto **Porte di ingresso** mostra tutti gli ingressi MIDI del
computer e se sono collegati. Vengono aperti solo i controller che hai
associato (alcuni sistemi danno una porta a un solo programma alla volta),
più tutti gli ingressi mentre fai apprendere un controllo; il programma non
ascolta mai le proprie porte. I controller si possono collegare e scollegare mentre il programma è in esecuzione:
ogni due secondi circa le porte vengono cercate di nuovo, e un controller
che ritorna viene collegato per nome, con le sue luci reimpostate.
**Riconnetti le porte** chiude ogni porta e la riapre senza toccare il cavo
(vedi [Risoluzione dei problemi](troubleshooting.md#a-midi-controller-does-not-respond)).

Su Linux il MIDI passa per ALSA: il tuo utente deve poter aprire il
sequencer (`/dev/snd/seq`, di solito facendo parte del gruppo `audio`).

## Associare un controllo {#binding-a-control}

Per ogni lettore c'è una riga per azione: **Play / Successivo**, **Pausa**,
**Stop**, **Stop in dissolvenza**, **Riavvia**, **Precedente**, **CUE** e
**Volume**.

1. Fai clic su **Apprendi** nella riga.
2. Premi il pulsante o muovi il fader che vuoi (nel frattempo dice **Muovi un
   controllo…**). Un pulsante accetta un tasto, un pad o un pulsante che
   invia un control change; **Volume** accetta un fader o una manopola (un
   control change) o un fader pitch-bend.
3. La riga mostra il dispositivo e il controllo, per esempio
   `APC mini · Note 36, channel 1`.

Se quel controllo era già associato a un'altra azione, passa a questa. `Esc`
o un nuovo clic sul pulsante annulla l'apprendimento. **Cancella** rimuove
un'associazione.

## Come si comportano i controlli {#how-the-controls-behave}

- Un pulsante agisce quando viene premuto (un tasto o un pad che va giù, o
  un control change che sale attraverso la metà del suo intervallo),
  esattamente come il pulsante del lettore sullo schermo. Un pulsante
  attenuato sullo schermo non fa nulla. Un pulsante **CUE** non fa nulla
  anche quando il suo preascolto si sentirebbe in onda mentre lì suona
  qualcosa: il MIDI non può rispondere alla domanda che pone lo schermo.
  Fermare un preascolto funziona sempre.
- Un fader muove il volume sulla stessa scala (la stessa legge da mixer) del fader sullo schermo. Per
  evitare salti, subentra solo quando raggiunge o supera il volume attuale
  (soft takeover); se il volume viene cambiato sullo schermo, il fader deve
  raggiungerlo di nuovo. Nulla cambia finché non muovi un controllo: avviare
  il programma non manda mai nulla in onda.
- Un'associazione di volume di un lettore la cui uscita Main è
  [bit-perfect](bit-perfect.md#what-it-costs) non fa nulla: il suo volume è
  mantenuto al 100 %. Quando l'uscita non è più bit-perfect, porta il
  controllo in cima per riprendere il volume.
- Play/Successivo e Precedente su un lettore del genere cambiano brano
  subito, senza dissolvenza incrociata. Le loro luci funzionano come al
  solito.
- Con **Accendi i pulsanti (feedback LED)** attivo, i pulsanti associati si
  accendono: Play mentre il lettore è in onda, Pausa che lampeggia mentre è
  in pausa, CUE durante il preascolto, e Stop, Stop in dissolvenza, Riavvia
  e Precedente finché possono agire. Le luci vanno alla porta di uscita del
  controller con lo stesso nome; se ne può impostare una diversa in
  `config.json` (`midi.devices`).
