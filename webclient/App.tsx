import React, { useEffect, useState } from "react";
import { render } from "react-dom";
const sseClientId = Date.now();
function App() {
    const [messages, setMessages] = useState<string[]>([]);

    useEffect(() => {
        var source = new EventSource(`http://localhost:8090/sse/clients/${sseClientId}/events`);
        const notify = (e: MessageEvent) => setMessages(prevMessages => [e.data, ...prevMessages]);
        source.addEventListener('message', notify);
        return () => source.removeEventListener('message', notify);
    }, [])
    return (<div>
        ClientID:{sseClientId}
        <ul>{messages.map((msg, i) => <li key={i}>{msg}</li>)}</ul>
    </div>);
}
render(<App />, document.getElementById('app'));