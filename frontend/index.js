fetch("socket_port").then(response => {
    if (response.ok) {
        response.text().then(port => {
            const socket = new WebSocket('ws://localhost:' + port);
        
            socket.onmessage = (event) => {
                handle_serial_data(event.data);
            };
        });
    } else {
        alert('could not get web socket port');
    }
});

function handle_serial_data(data) {
    let json = JSON.parse(data);
    console.log(json["ecu1"]);
    updateBar("ecu1", json["ecu1"]);
    updateBar("ecu2", json["ecu2"]);

}

// Calculate percentage height
function updateBar(barId, value) {
    const bar = document.getElementById(barId);
    if (!bar || bar.tagName.toLowerCase() != "data-bar") {
        console.log("data bar not found or is not a data bar class");
        return;
    }
    bar.updateValue(value);
}


class DataBar extends HTMLElement {
    static VALUE_LABEL_THRESHOLD = 20;
    constructor() {
        super();
        this.max = 0;
        this.min = 0;
        this.value = 0;
        this.fixed = false;
        this.name = "bar"
    }

    connectedCallback() {
        if (this.hasAttribute("fixed")) {
            this.fixed = this.getAttribute("fixed");
        }
        if (this.hasAttribute("min")) {
            this.min = Number(this.getAttribute("min"));
        }
        if (this.hasAttribute("max")) {
            this.max = Number(this.getAttribute("max"));
        }
        if (this.hasAttribute("value")) {
            this.value = Number(this.getAttribute("value"));
        }
        if (this.hasAttribute("name")) {
            this.name = this.getAttribute("name");
        }
        let percent = ((this.value - this.min) / (this.max - this.min)) * 100;

        this.innerHTML = `
            <p class="data-bar-label max">${this.max}</p>
            <div class="data-bar-outline">
                <div class="data-bar-fill" style="height: ${percent}%">
                    <p class="data-bar-label value-over" style="display:none">${this.value}</p>
                </div>
                <p class="data-bar-label value-under">${this.value}</p>
            </div>
            <p class="data-bar-label min">${this.min}</p>
            <p class="data-bar-label">${this.name}</p>
        `;
    }

    updateValue(value) {
        const barFill = this.querySelector('.data-bar-fill');
        const barValueOverLabel = this.querySelector('.data-bar-label.value-over');
        const barValueUnderLabel = this.querySelector('.data-bar-label.value-under');
        const barMaxLabel = this.querySelector('.data-bar-label.max');
        const barMinLabel = this.querySelector('.data-bar-label.min');

        if (!barFill) {
            console.error("data-bar-fill element not found");
            return;
        }
        this.value = value
        if (this.fixed) {
            // no-op
        } else if (value > this.max) {
            this.max = value;
            this.setAttribute("max", value);
            barMaxLabel.textContent = this.max;
        } else if (value < this.min) {
            this.min = value;
            this.setAttribute("min", value);
            barMinLabel.textContent = this.min;
        }

        const percent = ((value - this.min) / (this.max - this.min)) * 100;
        if (percent < DataBar.VALUE_LABEL_THRESHOLD) {
            barValueOverLabel.style.display = "none";
            barValueUnderLabel.style.display = "block";
        } else if (percent > 100 - DataBar.VALUE_LABEL_THRESHOLD) {
            barValueOverLabel.style.display = "block";
            barValueUnderLabel.style.display = "none";
        }
        this.setAttribute("value", value);
        barFill.style.height = percent + "%";
        barValueOverLabel.textContent = value;
        barValueUnderLabel.textContent = value;
    }
}

customElements.define('data-bar', DataBar);