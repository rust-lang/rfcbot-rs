import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';
import d3 from 'npm:d3';
import cloud from 'npm:d3-cloud';

export default Ember.Route.extend({
  model: function() {
    const url = `${ENV.apiBaseURL}hot-issues`;
    return Ember.$.getJSON(url)
      .then(resp => {
        const words = resp.word_counts.sort((a, b) => (b[1] < a[1]) ? -1 : ((b[1] > a[1]) ? 1 : 0));

        Ember.run.scheduleOnce('afterRender', this, function() {
          console.log('rendering word cloud');

          var fill = d3.scaleOrdinal(d3.schemeCategory20c);

          const width = document.getElementById("wordCloud")
            .clientWidth;

          var layout = cloud()
            .size([width, 500])
            .words(words.map(function(d) {
              return { text: d[0], size: d[1], test: "haha" };
            }))
            .padding(5)
            .rotate(function() {
              return 0;
            })
            .font("sans-serif")
            .fontSize(function(d) {
              return Math.sqrt(d.size * 1.5) * 2;
            })
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
              .style("font-size", function(d) {
                return d.size + "px";
              })
              .style("font-family", "sans-serif")
              .style("fill", function(d, i) {
                return fill(i);
              })
              .attr("text-anchor", "middle")
              .attr("transform", function(d) {
                return "translate(" + [d.x, d.y] + ")rotate(" + d.rotate + ")";
              })
              .text(function(d) {
                return d.text;
              });
          }
        });

        return {
          issues: resp.issues,
          word_counts: words
        };
      });
  }
});
