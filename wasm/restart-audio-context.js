// taken from https://developer.chrome.com/blog/web-audio-autoplay/#moving-forward
(function () {
    // An array of all contexts to resume on the page
    const audioContextList = [];

    setInterval(() => {
        audioContextList = audioContextList.filter(context => context.state !== 'running');
        audioContextList.forEach(context => {
            context.resume();
        });
    }, 100);
})();
