// The login screen of Willwitcher-OS Kavat Edition: what happens on it.
//
// The greeter gives the page a `lightdm` object: who can log in, which sessions there are,
// and the steps of logging in. Asking for a person starts a conversation: LightDM asks for
// the password with show_prompt, the page answers with respond, and authentication_complete
// says how it went.

(() => {
    "use strict";

    const TEXTS = {
        session: "Session", keyboard: "Keyboard", wrong: "That password is not right",
        suspend: "Suspend", restart: "Restart", shutdown: "Switch off", battery: "Battery",
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
        asked: false,     // LightDM is waiting for the password
        typed: null,      // a password entered before LightDM asked for it
    };

    function clock() {
        const now = new Date();
        $("clock").textContent = now.toLocaleTimeString(state.locale, { hour: "2-digit", minute: "2-digit", hour12: false });
        $("date").textContent = now.toLocaleDateString(state.locale, { weekday: "long", month: "long", day: "numeric" });
    }

    function picture(person) {
        if (person.image) {
            const image = document.createElement("img");
            image.src = "web-greeter://app" + person.image;
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

    // Starts the conversation with LightDM for a person, and shows them.
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

        const own = state.sessions.find((session) => session.key === person.session);
        if (own) pick_session(own);

        lightdm.cancel_authentication();
        lightdm.authenticate(person.username);
        $("password").focus();
    }

    function pick_session(session) {
        state.session = session;
        $("session").textContent = `${state.texts.session} · ${session.name}`;
    }

    function show_layout(layout) {
        state.layout = layout;
        $("keyboard").textContent = `${state.texts.keyboard} · ${layout.name.replace(/\s+/g, " ").toUpperCase()}`;
    }

    function next(list, current) {
        return list[(list.indexOf(current) + 1) % list.length];
    }

    function answer() {
        const password = $("password").value;
        if (state.asked) {
            state.asked = false;
            $("form").classList.add("busy");
            lightdm.respond(password);
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
        const language = (lightdm.language && lightdm.language.code) || navigator.language || "en";
        state.locale = language.replace("_", "-").replace(/[.@].*$/, "") || "en-US";
        try { new Date().toLocaleDateString(state.locale); } catch { state.locale = "en-US"; }
    }

    function ready() {
        locale();
        clock();
        setInterval(clock, 1000);

        state.sessions = lightdm.sessions || [];
        state.session = state.sessions.find((session) => session.key === lightdm.default_session) || state.sessions[0];
        if (state.session) pick_session(state.session);
        $("session").hidden = state.sessions.length < 2;
        $("session").onclick = () => pick_session(next(state.sessions, state.session));

        // the keyboards to offer are the ones the greeter's settings name, not every one there is
        state.layouts = greeter_config.layouts || [];
        const current = lightdm.layout || {};
        if (state.layouts.length) show_layout(state.layouts.find((layout) => layout.name === current.name) || state.layouts[0]);
        $("keyboard").hidden = state.layouts.length < 2;
        $("keyboard").onclick = () => {
            show_layout(next(state.layouts, state.layout));
            lightdm.layout = state.layout;
        };

        for (const [id, able, act] of [
            ["suspend", lightdm.can_suspend, () => lightdm.suspend()],
            ["restart", lightdm.can_restart, () => lightdm.restart()],
            ["shutdown", lightdm.can_shutdown, () => lightdm.shutdown()],
        ]) {
            $(id).textContent = state.texts[id];
            $(id).hidden = !able;
            $(id).onclick = act;
        }

        const battery = lightdm.can_access_battery ? lightdm.battery_data : null;
        if (battery && battery.level !== undefined) {
            $("battery").textContent = `${state.texts.battery} · ${battery.level}%`;
            $("battery").hidden = false;
        }

        lightdm.show_prompt.connect(() => {
            state.asked = true;
            $("form").classList.remove("busy");
            if (state.typed !== null) {
                state.typed = null;
                answer();
            }
        });
        lightdm.show_message.connect((text) => say(text));
        lightdm.authentication_complete.connect(() => {
            if (lightdm.is_authenticated) {
                document.body.classList.add("leaving");
                setTimeout(() => lightdm.start_session(state.session ? state.session.key : ""), 250);
                return;
            }
            say(state.texts.wrong, true);
            $("form").classList.remove("wrong");
            void $("form").offsetWidth;   // so that the shake plays again
            $("form").classList.add("wrong");
            choose(state.person);
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
            if (event.key.length === 1 && document.activeElement !== $("password")) $("password").focus();
        });
        addEventListener("mouseup", () => $("password").focus());

        state.people = lightdm.users || [];
        const last = state.people.find((person) => person.username === lightdm.select_user_hint);
        if (state.people.length) choose(last || state.people[0]);
    }

    // Opened in a browser instead of the greeter, the page stands in for LightDM with one
    // made-up person, whose password is "kavat": enough to see the screen and try it.
    function stand_in() {
        const signal = () => {
            const heard = [];
            return { connect: (callback) => heard.push(callback), emit: (...values) => heard.forEach((callback) => callback(...values)) };
        };
        const fake = {
            language: { code: navigator.language }, default_session: "kavat", select_user_hint: "",
            sessions: [{ key: "kavat", name: "Willwitcher-OS Kavat Edition" }, { key: "hyprland", name: "Hyprland" }],
            layouts: [{ name: "us", short_description: "us" }, { name: "es", short_description: "es" }],
            layout: { name: "us" },
            users: [{ username: "willwitcher", display_name: "willwitcher", image: "", session: "" }],
            can_suspend: true, can_restart: true, can_shutdown: true, can_access_battery: false,
            is_authenticated: false,
            show_prompt: signal(), show_message: signal(), authentication_complete: signal(),
            cancel_authentication() {},
            authenticate() { setTimeout(() => fake.show_prompt.emit("Password:", 1), 100); },
            respond(password) {
                fake.is_authenticated = password === "kavat";
                setTimeout(() => fake.authentication_complete.emit(), 400);
            },
            start_session() {}, suspend() {}, restart() {}, shutdown() {},
        };
        window.lightdm = fake;
        window.greeter_config = { layouts: fake.layouts };
        if (new URLSearchParams(location.search).has("awake")) document.body.classList.add("awake");
        start();
    }

    const start = () => (document.body.classList.contains("beside") ? ready_beside() : ready());
    if (location.protocol === "web-greeter:") addEventListener("GreeterReady", start);
    else addEventListener("DOMContentLoaded", stand_in);
})();
