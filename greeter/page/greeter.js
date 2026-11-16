// The login screen of Willwitcher-OS Kavat Edition: what happens on it.
//
// kavat-greeter gives the page a `greeter` object: who can log in, which sessions there
// are, and the steps of logging in. Asking for a person starts a conversation: greetd asks
// for the password with `prompt`, the page answers with respond(), and `complete` says how
// it went.

(() => {
    "use strict";

    const TEXTS = {
        session: "Session", keyboard: "Keyboard", wrong: "That password is not right", who: "Your name",
        suspend: "Suspend", restart: "Restart", shutdown: "Switch off",
    };

    const $ = (id) => document.getElementById(id);
    const state = {
        texts: TEXTS,
        locale: "en-US",
        people: [],
        person: null,
        sessions: [],
        session: null,
        layouts: [],
        layout: null,
        clock12: false,   // the clock counts twelve hours
        ask_name: false,  // nobody is shown: the name is typed
        asked: false,     // greetd is waiting for the password
        typed: null,      // a password entered before greetd asked for it
    };

    function clock() {
        const now = new Date();
        $("clock").textContent = now.toLocaleTimeString(state.locale, { hour: state.clock12 ? "numeric" : "2-digit", minute: "2-digit", hour12: state.clock12 });
        $("date").textContent = now.toLocaleDateString(state.locale, { weekday: "long", month: "long", day: "numeric" });
    }

    function picture(person) {
        if (person.image) {
            const image = document.createElement("img");
            image.src = "file://" + person.image;
            image.alt = "";
            // a picture that cannot be read leaves the place to the logo
            image.onerror = () => image.replaceWith(logo());
            return image;
        }
        return logo();
    }

    function logo() {
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
        use.setAttribute("href", "#logo");
        svg.append(use);
        return svg;
    }

    function say(text, wrong) {
        $("message").textContent = text || "";
        $("message").classList.toggle("wrong", Boolean(wrong));
    }

    // Starts the conversation with greetd for a person, and shows them.
    function choose(person) {
        state.person = person;
        state.asked = false;
        state.typed = null;
        $("avatar").replaceChildren(picture(person));
        $("name").textContent = person.display_name || person.username;
        $("password").value = "";
        $("form").classList.remove("busy");

        const others = state.people.filter((other) => other !== person).map((other) => {
            const button = document.createElement("button");
            button.type = "button";
            button.title = other.display_name || other.username;
            if (other.image) button.append(picture(other));
            else button.textContent = (other.display_name || other.username).slice(0, 1).toUpperCase();
            button.onclick = () => { say(""); choose(other); };
            return button;
        });
        $("people").replaceChildren(...others);

        greeter.authenticate(person.username);
        $("password").focus();
    }

    function pick_session(session) {
        state.session = session;
        $("session").textContent = `${state.texts.session} · ${session.name}`;
    }

    function show_layout(layout) {
        $("keyboard").textContent = `${state.texts.keyboard} · ${layout.toUpperCase()}`;
    }

    function next(list, current) {
        return list[(list.indexOf(current) + 1) % list.length];
    }

    function answer() {
        const password = $("password").value;
        if (state.ask_name) {
            // the name is typed: the conversation with greetd starts when it is known
            const name = $("who").value.trim();
            if (!name) { $("who").focus(); return; }
            if (!state.person || state.person.username !== name) {
                state.person = { username: name, display_name: name, image: "" };
                state.asked = false;
                greeter.authenticate(name);
            }
        }
        if (state.asked) {
            state.asked = false;
            $("form").classList.add("busy");
            greeter.respond(password);
        } else {
            state.typed = password;
        }
    }

    // Another screen of the same computer: the clock over the picture, and nothing to fill in.
    function ready_beside() {
        locale();
        clock();
        setInterval(clock, 1000);
    }

    function locale() {
        const language = greeter.language || navigator.language || "en";
        state.locale = language.replace("_", "-").replace(/[.@].*$/, "") || "en-US";
        try { new Date().toLocaleDateString(state.locale); } catch { state.locale = "en-US"; }
    }

    function ready() {
        locale();
        state.clock12 = Boolean(greeter.clock12);
        clock();
        setInterval(clock, 1000);

        state.sessions = greeter.sessions || [];
        state.session = state.sessions.find((session) => session.key === greeter.default_session) || state.sessions[0];
        if (state.session) pick_session(state.session);
        $("session").hidden = state.sessions.length < 2;
        $("session").onclick = () => pick_session(next(state.sessions, state.session));

        // the keyboard: each press goes to the next layout, and the greeter says which it is
        const layouts = greeter.layouts || [];
        if (layouts.length) show_layout(layouts[0]);
        $("keyboard").hidden = layouts.length < 2;
        $("keyboard").onclick = () => greeter.next_layout(show_layout);

        for (const [id, act] of [
            ["suspend", () => greeter.suspend()],
            ["restart", () => greeter.restart()],
            ["shutdown", () => greeter.shutdown()],
        ]) {
            $(id).textContent = state.texts[id];
            $(id).hidden = false;
            $(id).onclick = act;
        }

        greeter.prompt.connect(() => {
            state.asked = true;
            $("form").classList.remove("busy");
            if (state.typed !== null) {
                state.typed = null;
                answer();
            }
        });
        greeter.message.connect((text) => say(text));
        greeter.complete.connect((right) => {
            if (right) {
                document.body.classList.add("leaving");
                setTimeout(() => greeter.start_session(state.session ? state.session.key : ""), 250);
                return;
            }
            say(state.texts.wrong, true);
            $("form").classList.remove("wrong");
            void $("form").offsetWidth;   // so that the shake plays again
            $("form").classList.add("wrong");
            if (state.ask_name) {
                state.person = null;
                state.asked = false;
                $("password").value = "";
                $("form").classList.remove("busy");
                $("password").focus();
            } else {
                choose(state.person);
            }
        });

        $("form").onsubmit = (event) => {
            event.preventDefault();
            say("");
            answer();
        };
        $("password").oninput = () => $("form").classList.remove("wrong");

        // The corners show while the pointer is at the bottom of the screen, and a while after.
        let asleep;
        addEventListener("mousemove", (event) => {
            if (event.clientY < innerHeight * 0.75) return;
            document.body.classList.add("awake");
            clearTimeout(asleep);
            asleep = setTimeout(() => document.body.classList.remove("awake"), 4000);
        });
        // Typing goes to the password, wherever the last click was.
        addEventListener("keydown", (event) => {
            if (event.key.length === 1 && document.activeElement !== $("password") && document.activeElement !== $("who")) $("password").focus();
        });
        addEventListener("mouseup", (event) => { if (event.target !== $("who")) $("password").focus(); });

        state.people = greeter.users || [];
        state.ask_name = Boolean(greeter.ask_name);
        if (state.ask_name) {
            for (const id of ["people", "avatar", "name"]) $(id).hidden = true;
            $("whobox").hidden = false;
            $("who").placeholder = state.texts.who;
            $("who").onkeydown = (event) => {
                if (event.key === "Enter") { event.preventDefault(); $("password").focus(); }
            };
            $("who").focus();
        } else if (state.people.length) {
            choose(state.people[0]);
        }
    }

    // Opened in a browser instead of the greeter, the page stands in for it with one made-up
    // person, whose password is "kavat": enough to see the screen and try it.
    function stand_in() {
        const signal = () => {
            const heard = [];
            return { connect: (callback) => heard.push(callback), emit: (...values) => heard.forEach((callback) => callback(...values)) };
        };
        let layout = 0;
        window.greeter = {
            language: navigator.language, default_session: "kavat", layouts: ["us", "es"],
            sessions: [{ key: "kavat", name: "Willwitcher-OS Kavat Edition" }, { key: "hyprland", name: "Hyprland" }],
            users: [{ username: "willwitcher", display_name: "willwitcher", image: "" }],
            prompt: signal(), message: signal(), complete: signal(),
            authenticate() { setTimeout(() => greeter.prompt.emit("Password:"), 100); },
            respond(password) { setTimeout(() => greeter.complete.emit(password === "kavat"), 400); },
            next_layout(tell) { layout = (layout + 1) % 2; tell(greeter.layouts[layout]); },
            start_session() {}, suspend() {}, restart() {}, shutdown() {},
        };
        if (new URLSearchParams(location.search).has("awake")) document.body.classList.add("awake");
        start();
    }

    const start = () => (document.body.classList.contains("beside") ? ready_beside() : ready());
    addEventListener("DOMContentLoaded", () => {
        if (!window.qt || !window.QWebChannel) return stand_in();
        new QWebChannel(qt.webChannelTransport, (channel) => {
            window.greeter = channel.objects.greeter;
            start();
        });
    });
})();
