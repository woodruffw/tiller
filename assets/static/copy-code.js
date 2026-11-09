document.querySelectorAll("pre code").forEach(codeEl => {
    const pre = codeEl.parentElement;
    const container = document.createElement("div");
    container.className = "code-container";
    const btn = document.createElement("button");
    btn.className = "copy-btn";
    btn.textContent = "Copy";
    btn.addEventListener("click", () => {
        navigator.clipboard.writeText(codeEl.textContent).then(() => {
            btn.textContent = "Copied!";
            btn.classList.add("copied");
            setTimeout(() => {
                btn.textContent = "Copy";
                btn.classList.remove("copied");
            }, 2000);
        });
    });
    pre.parentNode.insertBefore(container, pre);
    container.appendChild(pre);
    container.appendChild(btn);
});
