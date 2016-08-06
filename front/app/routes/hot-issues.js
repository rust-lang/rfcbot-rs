import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';
import d3 from 'd3';
import cloud from 'npm:d3-cloud';

export default Ember.Route.extend({
  model() {
    const url = `${ENV.apiBaseURL}hot-issues`;
    return fetch(url)
      .then(response => response.json())
      .then(({ word_counts, issues }) => { 
        const words = word_counts.sort((a, b) => (b[1] < a[1]) ? -1 : ((b[1] > a[1]) ? 1 : 0));

        Ember.run.scheduleOnce('afterRender', this, () => {
          console.log('rendering word cloud');

          const fill = d3.scaleOrdinal(d3.schemeCategory20c);

          const width = document.getElementById("wordCloud")
            .clientWidth;

          const layout = cloud()
            .size([width, 500])
            .words(words.map(d => ({ text: d[0], size: d[1], test: "haha" })))
            .padding(5)
            .rotate(() => 0)
            .font("sans-serif")
            .fontSize(d => Math.sqrt(d.size * 1.5) * 2)
            .on("end", draw);

          layout.start();

          function draw(words) {
            d3.select("#wordCloud")
              .append("svg")
              .attr("width", layout.size()[0])
              .attr("height", layout.size()[1])
              .append("g")
              .attr("transform", "translate(" + layout.size()[0] / 2 + "," + layout.size()[1] / 2 + ")")
              .selectAll("text")
              .data(words)
              .enter()
              .append("text")
              .style("font-size", d => d.size + "px")
              .style("font-family", "sans-serif")
              .style("fill", (d, i) => fill(i))
              .attr("text-anchor", "middle")
              .attr("transform", d => `translate(${[d.x, d.y]})rotate(${d.rotate})`)
              .text(d => d.text);
          }
        });

        return {
          issues: issues,
          word_counts: words
        };
      });
  }
});
