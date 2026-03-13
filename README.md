<div align="center">

# NextTabletDriver

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Version](https://img.shields.io/badge/Version-1.26.1303.03-orange?style=for-the-badge)
![Osu!](https://img.shields.io/badge/Osu!-Next%20Tablet%20Driver-pink?style=for-the-badge&logo=osu)

</div>

> [!IMPORTANT]
> **I'm currently looking for a definitive name!** If you have any cool suggestions, please feel free to open a discussion and let me know.

---

Welcome to my project! I'm building a high-performance, event-driven tablet driver in Rust, specifically tailored for **Osu!** and digital drawing. called Next Tablet Driver.. or NTD (or NextTD) yess..

## My Vision for the Project

I started this project because I wanted a lightweight, ultra-responsive alternative for tablet users. By building it from scratch in Rust and moving away from traditional polling to a modern **event-driven architecture**, I've been able to significantly reduce CPU overhead and improve the overall "feel" of the driver.

### Why use this over OpenTabletDriver?
OpenTabletDriver is amazing, but for my specific needs and maybe yours, I wanted something that felt even more integrated and performance-focused. This project is my take on how a modern driver should handle input.

## A Huge Thank You to OTD

I want to be very clear: **this project would not exist without OpenTabletDriver.** 

I rely heavily on their massive library of **JSON tablet configurations** and **parsers**. They did the impossible work of mapping out hundreds of tablets, and I am incredibly grateful to them for making that data open. Without their work, this project would have been impossible to start. (**really, thanks to them and their community**)

## Current State of the Project

Technically, we're still in the **prototyping phase**. I've tested the driver on several machines and with various tablets, but there's a lot of work left to do.

I'll be honest: **the code isn't perfect yet.** 

*   **HID Initialization**: It currently takes longer than I’d like, and I’m still looking for a way to optimize this better than my current implementation.
*   **Parser Structure**: I’ll admit it's a bit of a mess right now. I'm not entirely happy with the current architecture and I think it needs a complete rethink to be cleaner and more efficient.

## My Approach to AI

I developed a large part of this project with the help of AI (specifically Claude Opus/Sonnet and Gemini 3 Pro/Flash). 

I see AI as one of the best tools in the world when used correctly. Building a project like this is long and tedious, and AI helps speed up that process tremendously. However, I want to emphasize that **this is not "Vibe-Coding."**

I act as the **architect, supervisor, and lead tester**. I have personally reviewed and corrected every single line of code. If a piece of code is in here, it's because I've vetted it and ensured it works. I'm not just copy-pasting; I'm using AI to build exactly what I envision.

## Want to Help?

I'm very open to contributions! If you're a Rust developer and want to dive in, I'd love the help. 

**My priorities right now:**
1.  **Speeding up HID initialization.**
2.  **Cleaning up the parser logic.**
3.  **General refactoring** of the prototyping code.

**On AI for contributors:**
I have no problem with you using AI for your contributions I use it myself! All I ask is that you **understand what you're submitting**. Don't just paste code without knowing what it does. If you don't fully understand a part of your contribution but it works, just be honest and mention it! I'd rather have an honest contribution I can help refine than a blind one.
