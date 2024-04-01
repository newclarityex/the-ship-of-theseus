// taken from https://developer.chrome.com/blog/web-audio-autoplay/#moving-forward
(function () {
    // An array of all contexts to resume on the page
    const audioContextList = [];

    // A proxy object to intercept AudioContexts and
    // add them to the array for tracking and resuming later
    self.AudioContext = new Proxy(self.AudioContext, {
        construct(target, args) {
            const result = new target(...args);
            audioContextList.push(result);
            return result;
        },
    });

    setInterval(() => {
        audioContextList = audioContextList.filter(context => context.state !== 'running');
        audioContextList.forEach(context => {
            context.resume();
        });
    }, 100);
})();
