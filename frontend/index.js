fetch("socket_port").then(response => {
    if (response.ok) {
        response.text().then(port => {
            const socket = new WebSocket(`ws://${location.hostname}:${port}`);
        
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
    for (const key in json) {
        update(key, json[key]);
    }
    // update("ecu1", json["ecu1"]);
}

// Calculate percentage height
function update(dataId, value) {
    const dataE = document.getElementById(dataId);
    if (!dataE || !(dataE instanceof DataElement)) {
        console.log(`DataElement with id ${dataId} not found`);
        return;
    }
    dataE.updateValue(value);
}

class DataElement extends HTMLElement {
    constructor() {
        super();
        this.max = 0;
        this.min = 0;
        this.value = 0;
        this.fixed = false;
        this.name = "data";

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
    }

    updateValue(value) {
        this.value = value
        this.setAttribute("value", value);
        if (this.fixed) {
            // no-op
        } else if (value > this.max) {
            this.max = value;
            this.setAttribute("max", value);
        } else if (value < this.min) {
            this.min = value;
            this.setAttribute("min", value);
        }
        this.updateData();
    }
}

class DataBar extends DataElement {
    static VALUE_LABEL_THRESHOLD = 20;

    connectedCallback() {
        let percent = ((this.value - this.min) / (this.max - this.min)) * 100;

        this.innerHTML = `
            <p class="data-label max">${this.max}</p>
            <div class="data-bar-outline">
                <div class="data-bar-fill" style="height: ${percent}%">
                    <p class="data-label value-over" style="display:none">${this.value}</p>
                </div>
                <p class="data-label value-under">${this.value}</p>
            </div>
            <p class="data-label min">${this.min}</p>
            <p class="data-label">${this.name}</p>
        `;
    }

    updateData() {
        const barFill = this.querySelector('.data-bar-fill');
        const barValueOverLabel = this.querySelector('.data-label.value-over');
        const barValueUnderLabel = this.querySelector('.data-label.value-under');
        const barMaxLabel = this.querySelector('.data-label.max');
        const barMinLabel = this.querySelector('.data-label.min');
        barMinLabel.textContent = this.min;
        barMaxLabel.textContent = this.max;

        const percent = ((this.value - this.min) / (this.max - this.min)) * 100;
        if (percent < DataBar.VALUE_LABEL_THRESHOLD) {
            barValueOverLabel.style.display = "none";
            barValueUnderLabel.style.display = "block";
        } else if (percent > 100 - DataBar.VALUE_LABEL_THRESHOLD) {
            barValueOverLabel.style.display = "block";
            barValueUnderLabel.style.display = "none";
        }
        barFill.style.height = percent + "%";
        barValueOverLabel.textContent = this.value;
        barValueUnderLabel.textContent = this.value;
    }
}

customElements.define('data-bar', DataBar);

class DataGauge extends DataElement {
    static ANGLE_OFFSET = 30;

    connectedCallback() {
        this.innerHTML = `
            <div class="data-gauge">
                <div class="data-gauge-bg"> 
                    <div class="data-gauge-needle"></div>
                    <div class="data-gauge-needle mark-container" style="transform: rotate(${-180 + DataGauge.ANGLE_OFFSET}deg)">
                        <div class="data-gauge-needle mark min"></div>
                    </div>
                    <div class="data-gauge-needle mark-container" style="transform: rotate(${180 - DataGauge.ANGLE_OFFSET}deg)">
                        <div class="data-gauge-needle mark max"></div>
                    </div>
                    <div class="data-gauge-needle mark-container" style="transform: rotate(${(360 - 2 * DataGauge.ANGLE_OFFSET)/4}deg)">
                        <div class="data-gauge-needle mark"></div>
                    </div>
                    <div class="data-gauge-needle mark-container" style="transform: rotate(${-(360 - 2 * DataGauge.ANGLE_OFFSET)/4}deg)">
                        <div class="data-gauge-needle mark"></div>
                    </div>
                    <div class="data-gauge-needle mark-container">
                        <div class="data-gauge-needle mark"></div>
                    </div>
                    <p class="data-label gauge">${this.value}</p>
                </div>
                <div class="data-gauge-min-max">
                    <p class="data-label min">${this.min}</p>           
                    <p class="data-label max">${this.max}</p>
                </div>
                <p class="data-label">${this.name}</p>
            </div>

        `;
    }

    updateData() {
        const needle = this.querySelector('.data-gauge-needle');
        const minLabel = this.querySelector('.data-label.min');
        const maxLabel = this.querySelector('.data-label.max');
        const valueLabel = this.querySelector('.data-label.gauge');
        minLabel.textContent = this.min;
        maxLabel.textContent = this.max;
        valueLabel.textContent = this.value;

        const angle = ((this.value - this.min) / (this.max - this.min)) * (360 - 2 * DataGauge.ANGLE_OFFSET) - 180 + DataGauge.ANGLE_OFFSET;

        needle.style.transform = `rotate(${angle}deg)`;
    }
}

customElements.define("data-gauge", DataGauge);